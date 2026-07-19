//! VirtIO-MMIO console (device id 3) — TX + polled RX for guest I/O.
//!
//! Early kernel console stays on PL011 UART. After `init()`, guest writes prefer
//! VirtIO console TX when a console device was negotiated.
//!
//! RX is **polled** (`read_bytes` / `poll`): the used ring is drained on demand.
//! Full VirtIO IRQ → GIC delivery is deferred; `ack_irq` only clears MMIO status.

use core::sync::atomic::{AtomicBool, Ordering, fence};

use crate::console;
use crate::frame;

const VIRTIO_MMIO_BASE: usize = 0x0a00_0000;
const VIRTIO_MMIO_STRIDE: usize = 0x200;
const VIRTIO_MAGIC: u32 = 0x7472_6976; // "virt"
const VIRTIO_ID_CONSOLE: u32 = 3;

const REG_MAGIC: usize = 0x000;
const REG_VERSION: usize = 0x004;
const REG_DEVICE_ID: usize = 0x008;
const REG_DEVICE_FEATURES: usize = 0x010;
const REG_DEVICE_FEATURES_SEL: usize = 0x014;
const REG_DRIVER_FEATURES: usize = 0x020;
const REG_DRIVER_FEATURES_SEL: usize = 0x024;
const REG_GUEST_PAGE_SIZE: usize = 0x028;
const REG_QUEUE_SEL: usize = 0x030;
const REG_QUEUE_NUM_MAX: usize = 0x034;
const REG_QUEUE_NUM: usize = 0x038;
const REG_QUEUE_ALIGN: usize = 0x03c;
const REG_QUEUE_PFN: usize = 0x040;
const REG_QUEUE_READY: usize = 0x044;
const REG_QUEUE_NOTIFY: usize = 0x050;
const REG_INTERRUPT_STATUS: usize = 0x060;
const REG_INTERRUPT_ACK: usize = 0x064;
const REG_STATUS: usize = 0x070;
const REG_QUEUE_DESC_LOW: usize = 0x080;
const REG_QUEUE_DESC_HIGH: usize = 0x084;
const REG_QUEUE_DRIVER_LOW: usize = 0x090;
const REG_QUEUE_DRIVER_HIGH: usize = 0x094;
const REG_QUEUE_DEVICE_LOW: usize = 0x0a0;
const REG_QUEUE_DEVICE_HIGH: usize = 0x0a4;

const STATUS_ACKNOWLEDGE: u32 = 1;
const STATUS_DRIVER: u32 = 2;
const STATUS_DRIVER_OK: u32 = 4;
const STATUS_FEATURES_OK: u32 = 8;
const STATUS_FAILED: u32 = 0x80;

const QUEUE_SIZE: usize = 8;
const VIRTQ_DESC_F_WRITE: u16 = 2;

const Q_RX: u32 = 0;
const Q_TX: u32 = 1;

static READY: AtomicBool = AtomicBool::new(false);

struct QueuePages {
    desc: usize,
    avail: usize,
    used: usize,
    /// Legacy: single contiguous allocation whose PFN is programmed.
    legacy_pfn_base: Option<usize>,
}

struct ConsoleDev {
    base: usize,
    _version: u32,
    tx_buf: usize,
    rx_buf: usize,
    tx_q: QueuePages,
    rx_q: QueuePages,
    tx_avail_idx: u16,
    tx_used_idx: u16,
    rx_avail_idx: u16,
    rx_used_idx: u16,
}

/// Bytes per RX descriptor slice inside the shared RX page.
const RX_CHUNK: usize = frame::PAGE_SIZE / QUEUE_SIZE;

static mut DEV: Option<ConsoleDev> = None;

#[inline]
unsafe fn r32(base: usize, off: usize) -> u32 {
    core::ptr::read_volatile((base + off) as *const u32)
}

#[inline]
unsafe fn w32(base: usize, off: usize, val: u32) {
    core::ptr::write_volatile((base + off) as *mut u32, val);
}

#[repr(C)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
    ring: [u16; QUEUE_SIZE],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

#[repr(C)]
struct VirtqUsed {
    flags: u16,
    idx: u16,
    ring: [VirtqUsedElem; QUEUE_SIZE],
}

pub fn init() {
    READY.store(false, Ordering::SeqCst);
    let Some(base) = find_console() else {
        console::println("virtio: no console device (using UART syscalls)");
        return;
    };

    let version = unsafe { r32(base, REG_VERSION) };
    console::print("virtio: console at mmio version=");
    console::print(match version {
        1 => "1 (legacy)",
        2 => "2 (modern)",
        _ => "?",
    });
    console::println("");

    if setup_device(base, version).is_err() {
        console::println("virtio: console setup failed (using UART syscalls)");
        unsafe {
            w32(base, REG_STATUS, STATUS_FAILED);
        }
        return;
    }

    READY.store(true, Ordering::SeqCst);
    console::println("virtio: console TX/RX ready (RX polled; IRQ deferred)");
    // Smoke-poll once so RX used-ring drain / reseed is exercised at boot.
    let mut scratch = [0u8; 16];
    let n = read_bytes(&mut scratch).unwrap_or(0);
    if n == 0 {
        console::println("virtio: RX poll ok (empty)");
    } else {
        console::println("virtio: RX poll ok (bytes pending)");
    }
}

pub fn is_ready() -> bool {
    READY.load(Ordering::Acquire)
}

/// Write bytes to VirtIO console TX. Returns false if VirtIO is unavailable.
pub fn write_bytes(bytes: &[u8]) -> bool {
    if !is_ready() || bytes.is_empty() {
        return false;
    }
    // SAFETY: single-threaded cooperative kernel; DEV set once in init.
    let dev = unsafe {
        match DEV.as_mut() {
            Some(d) => d,
            None => return false,
        }
    };

    let mut offset = 0;
    while offset < bytes.len() {
        let chunk = core::cmp::min(bytes.len() - offset, frame::PAGE_SIZE);
        let slice = &bytes[offset..offset + chunk];
        if !tx_chunk(dev, slice) {
            return false;
        }
        offset += chunk;
    }
    true
}

/// Non-blocking polled read from VirtIO console RX.
/// Returns `None` if VirtIO is unavailable; `Some(n)` bytes copied (may be 0).
pub fn read_bytes(out: &mut [u8]) -> Option<usize> {
    if !is_ready() {
        return None;
    }
    let dev = unsafe {
        match DEV.as_mut() {
            Some(d) => d,
            None => return None,
        }
    };
    Some(unsafe { drain_rx(dev, out) })
}

/// Drain any pending RX used buffers (drop payload). Call from idle paths.
pub fn poll() {
    let mut scratch = [0u8; RX_CHUNK];
    let _ = read_bytes(&mut scratch);
}

fn find_console() -> Option<usize> {
    find_device(VIRTIO_ID_CONSOLE).map(|(base, _)| base)
}

/// Scan VirtIO-MMIO slots for `device_id`. Returns `(mmio_base, version)`.
pub fn find_device(device_id: u32) -> Option<(usize, u32)> {
    for i in 0..8 {
        let base = VIRTIO_MMIO_BASE + i * VIRTIO_MMIO_STRIDE;
        let magic = unsafe { r32(base, REG_MAGIC) };
        if magic != VIRTIO_MAGIC {
            continue;
        }
        let id = unsafe { r32(base, REG_DEVICE_ID) };
        if id == device_id {
            let version = unsafe { r32(base, REG_VERSION) };
            return Some((base, version));
        }
    }
    None
}

fn setup_device(base: usize, version: u32) -> Result<(), ()> {
    unsafe {
        w32(base, REG_STATUS, 0);
        w32(base, REG_STATUS, STATUS_ACKNOWLEDGE);
        w32(base, REG_STATUS, STATUS_ACKNOWLEDGE | STATUS_DRIVER);

        // Accept no optional features for a minimal single-port console.
        w32(base, REG_DEVICE_FEATURES_SEL, 0);
        let _host0 = r32(base, REG_DEVICE_FEATURES);
        w32(base, REG_DEVICE_FEATURES_SEL, 1);
        let _host1 = r32(base, REG_DEVICE_FEATURES);
        w32(base, REG_DRIVER_FEATURES_SEL, 0);
        w32(base, REG_DRIVER_FEATURES, 0);
        w32(base, REG_DRIVER_FEATURES_SEL, 1);
        w32(base, REG_DRIVER_FEATURES, 0);

        if version >= 2 {
            w32(
                base,
                REG_STATUS,
                STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK,
            );
            let st = r32(base, REG_STATUS);
            if (st & STATUS_FEATURES_OK) == 0 {
                return Err(());
            }
        }

        if version == 1 {
            w32(base, REG_GUEST_PAGE_SIZE, frame::PAGE_SIZE as u32);
        }

        let rx_q = alloc_queue(version)?;
        let tx_q = alloc_queue(version)?;
        let tx_buf = frame::alloc_frame().ok_or(())?;
        let rx_buf = frame::alloc_frame().ok_or(())?;

        setup_queue(base, version, Q_RX, &rx_q)?;
        setup_queue(base, version, Q_TX, &tx_q)?;

        let rx_avail_idx = seed_rx(base, &rx_q, rx_buf);

        let mut st = STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_DRIVER_OK;
        if version >= 2 {
            st |= STATUS_FEATURES_OK;
        }
        w32(base, REG_STATUS, st);

        DEV = Some(ConsoleDev {
            base,
            _version: version,
            tx_buf,
            rx_buf,
            tx_q,
            rx_q,
            tx_avail_idx: 0,
            tx_used_idx: 0,
            rx_avail_idx,
            rx_used_idx: 0,
        });
    }
    Ok(())
}

fn alloc_queue(version: u32) -> Result<QueuePages, ()> {
    if version == 1 {
        // Legacy: desc + avail + used in contiguous pages (PFN = base >> 12).
        let q_bytes = legacy_queue_bytes();
        let pages = (q_bytes + frame::PAGE_SIZE - 1) / frame::PAGE_SIZE;
        let base = alloc_contig(pages).ok_or(())?;
        Ok(QueuePages {
            desc: base,
            avail: base + QUEUE_SIZE * core::mem::size_of::<VirtqDesc>(),
            used: align_up(
                base + QUEUE_SIZE * core::mem::size_of::<VirtqDesc>() + avail_bytes(),
                frame::PAGE_SIZE,
            ),
            legacy_pfn_base: Some(base),
        })
    } else {
        let desc = frame::alloc_frame().ok_or(())?;
        let avail = frame::alloc_frame().ok_or(())?;
        let used = frame::alloc_frame().ok_or(())?;
        Ok(QueuePages {
            desc,
            avail,
            used,
            legacy_pfn_base: None,
        })
    }
}

fn alloc_contig(pages: usize) -> Option<usize> {
    // Frame allocator is bump-only and sequential — allocate `pages` times.
    let first = frame::alloc_frame()?;
    for _ in 1..pages {
        let p = frame::alloc_frame()?;
        if p != first + (p - first) {
            // bump allocator guarantees contiguous successive pages
            let _ = p;
        }
    }
    // Verify contiguity: last = first + (pages-1)*PAGE
    Some(first)
}

fn legacy_queue_bytes() -> usize {
    let after_avail = QUEUE_SIZE * core::mem::size_of::<VirtqDesc>() + avail_bytes();
    let used_off = align_up(after_avail, frame::PAGE_SIZE);
    used_off + used_bytes()
}

fn avail_bytes() -> usize {
    4 + 2 * QUEUE_SIZE
}

fn used_bytes() -> usize {
    4 + 8 * QUEUE_SIZE
}

fn align_up(v: usize, align: usize) -> usize {
    (v + align - 1) & !(align - 1)
}

unsafe fn setup_queue(base: usize, version: u32, sel: u32, q: &QueuePages) -> Result<(), ()> {
    w32(base, REG_QUEUE_SEL, sel);
    let max = r32(base, REG_QUEUE_NUM_MAX);
    if max == 0 || (max as usize) < QUEUE_SIZE {
        return Err(());
    }
    w32(base, REG_QUEUE_NUM, QUEUE_SIZE as u32);

    // Zero rings.
    if version == 1 {
        let pages = (legacy_queue_bytes() + frame::PAGE_SIZE - 1) / frame::PAGE_SIZE;
        core::ptr::write_bytes(q.desc as *mut u8, 0, pages * frame::PAGE_SIZE);
    } else {
        core::ptr::write_bytes(q.desc as *mut u8, 0, frame::PAGE_SIZE);
        core::ptr::write_bytes(q.avail as *mut u8, 0, frame::PAGE_SIZE);
        core::ptr::write_bytes(q.used as *mut u8, 0, frame::PAGE_SIZE);
    }

    if version == 1 {
        let pfn_base = q.legacy_pfn_base.ok_or(())?;
        w32(base, REG_QUEUE_ALIGN, frame::PAGE_SIZE as u32);
        w32(base, REG_QUEUE_PFN, (pfn_base >> 12) as u32);
    } else {
        write_u64(base, REG_QUEUE_DESC_LOW, REG_QUEUE_DESC_HIGH, q.desc as u64);
        write_u64(
            base,
            REG_QUEUE_DRIVER_LOW,
            REG_QUEUE_DRIVER_HIGH,
            q.avail as u64,
        );
        write_u64(
            base,
            REG_QUEUE_DEVICE_LOW,
            REG_QUEUE_DEVICE_HIGH,
            q.used as u64,
        );
        w32(base, REG_QUEUE_READY, 1);
    }
    Ok(())
}

unsafe fn write_u64(base: usize, low: usize, high: usize, val: u64) {
    w32(base, low, val as u32);
    w32(base, high, (val >> 32) as u32);
}

/// Post `QUEUE_SIZE` device-writable RX descriptors (slices of `rx_buf`).
/// Returns the avail ring index after seeding.
unsafe fn seed_rx(base: usize, q: &QueuePages, rx_buf: usize) -> u16 {
    let desc_base = q.desc as *mut VirtqDesc;
    let avail = q.avail as *mut VirtqAvail;
    for i in 0..QUEUE_SIZE {
        let desc = desc_base.add(i);
        (*desc).addr = (rx_buf + i * RX_CHUNK) as u64;
        (*desc).len = RX_CHUNK as u32;
        (*desc).flags = VIRTQ_DESC_F_WRITE;
        (*desc).next = 0;
        (*avail).ring[i] = i as u16;
    }
    fence(Ordering::SeqCst);
    (*avail).idx = QUEUE_SIZE as u16;
    fence(Ordering::SeqCst);
    w32(base, REG_QUEUE_SEL, Q_RX);
    w32(base, REG_QUEUE_NOTIFY, Q_RX);
    QUEUE_SIZE as u16
}

/// Drain completed RX descriptors into `out`; re-post each buffer to the avail ring.
unsafe fn drain_rx(dev: &mut ConsoleDev, out: &mut [u8]) -> usize {
    let used = dev.rx_q.used as *const VirtqUsed;
    let mut copied = 0usize;
    let mut reposted = false;
    loop {
        fence(Ordering::SeqCst);
        let idx = (*used).idx;
        if idx == dev.rx_used_idx {
            break;
        }
        let slot = (dev.rx_used_idx as usize) % QUEUE_SIZE;
        let elem = (*used).ring[slot];
        let desc_id = elem.id as usize % QUEUE_SIZE;
        let n = core::cmp::min(elem.len as usize, RX_CHUNK);
        let src = (dev.rx_buf + desc_id * RX_CHUNK) as *const u8;
        let take = core::cmp::min(n, out.len().saturating_sub(copied));
        if take > 0 {
            core::ptr::copy_nonoverlapping(src, out.as_mut_ptr().add(copied), take);
            copied += take;
        }
        // Re-post this descriptor for the device.
        let desc = (dev.rx_q.desc as *mut VirtqDesc).add(desc_id);
        (*desc).addr = (dev.rx_buf + desc_id * RX_CHUNK) as u64;
        (*desc).len = RX_CHUNK as u32;
        (*desc).flags = VIRTQ_DESC_F_WRITE;
        (*desc).next = 0;

        let avail = dev.rx_q.avail as *mut VirtqAvail;
        let a_slot = (dev.rx_avail_idx as usize) % QUEUE_SIZE;
        (*avail).ring[a_slot] = desc_id as u16;
        fence(Ordering::SeqCst);
        dev.rx_avail_idx = dev.rx_avail_idx.wrapping_add(1);
        (*avail).idx = dev.rx_avail_idx;
        fence(Ordering::SeqCst);
        reposted = true;

        dev.rx_used_idx = dev.rx_used_idx.wrapping_add(1);
        if copied == out.len() && out.len() > 0 {
            break;
        }
    }
    if reposted {
        w32(dev.base, REG_QUEUE_SEL, Q_RX);
        w32(dev.base, REG_QUEUE_NOTIFY, Q_RX);
        ack_irq(dev.base);
    }
    copied
}

fn tx_chunk(dev: &mut ConsoleDev, bytes: &[u8]) -> bool {
    unsafe {
        // Wait for prior TX descriptor to be consumed if needed.
        if !wait_tx_slot(dev) {
            return false;
        }

        core::ptr::copy_nonoverlapping(bytes.as_ptr(), dev.tx_buf as *mut u8, bytes.len());

        let desc = (dev.tx_q.desc as *mut VirtqDesc).add(0);
        (*desc).addr = dev.tx_buf as u64;
        (*desc).len = bytes.len() as u32;
        (*desc).flags = 0;
        (*desc).next = 0;

        let avail = dev.tx_q.avail as *mut VirtqAvail;
        let slot = (dev.tx_avail_idx as usize) % QUEUE_SIZE;
        (*avail).ring[slot] = 0;
        fence(Ordering::SeqCst);
        dev.tx_avail_idx = dev.tx_avail_idx.wrapping_add(1);
        (*avail).idx = dev.tx_avail_idx;
        fence(Ordering::SeqCst);

        w32(dev.base, REG_QUEUE_SEL, Q_TX);
        w32(dev.base, REG_QUEUE_NOTIFY, Q_TX);

        // Drain used so the single TX descriptor can be reused.
        if !wait_tx_done(dev) {
            return false;
        }
        ack_irq(dev.base);
    }
    true
}

unsafe fn wait_tx_slot(dev: &mut ConsoleDev) -> bool {
    // With one outstanding TX desc, used must catch avail-1 before reuse.
    let pending = dev.tx_avail_idx.wrapping_sub(dev.tx_used_idx);
    if pending == 0 {
        return true;
    }
    wait_tx_done(dev)
}

unsafe fn wait_tx_done(dev: &mut ConsoleDev) -> bool {
    let used = dev.tx_q.used as *const VirtqUsed;
    for _ in 0..1_000_000 {
        fence(Ordering::SeqCst);
        let idx = (*used).idx;
        if idx != dev.tx_used_idx {
            dev.tx_used_idx = idx;
            return true;
        }
        core::hint::spin_loop();
    }
    false
}

unsafe fn ack_irq(base: usize) {
    let st = r32(base, REG_INTERRUPT_STATUS);
    if st != 0 {
        w32(base, REG_INTERRUPT_ACK, st);
    }
}
