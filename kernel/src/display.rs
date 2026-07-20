//! Display foundations: VirtIO-GPU scanout + QEMU ramfb smoke test.
//!
//! Sprint 5 (SCRUM-29): scan VirtIO-MMIO for GPU (device id 16), and when QEMU
//! exposes `etc/ramfb` via fw_cfg, map a 480×800 XRGB8888 surface, solid-fill,
//! and draw a few text glyphs (matches host `userspace/shell` visual contract).
//!
//! Sprint 8 (SCRUM-42): when a VirtIO-GPU device is present, negotiate the
//! **control queue**, create a 2D resource, SET_SCANOUT, transfer + flush a
//! checkerboard solid-fill (kernel EL1). **ramfb remains the default visible
//! path** for `run-qemu-gui.ps1` without `-VirtioGpu`.
//!
//! Ramfb is activated only by a fw_cfg **DMA write** of `RAMFBCfg` to `etc/ramfb`.
//! Byte stores to the fw_cfg DATA register are ignored (QEMU ≥ 2.4); without DMA
//! the guest can paint RAM and log success while the host window stays on
//! "Guest has not initialized the display (yet)".

use crate::console;
use crate::frame;
use crate::virtio;

/// Mobile-shaped surface (same as host PPM sketch).
const FB_WIDTH: u32 = 480;
const FB_HEIGHT: u32 = 800;
const FB_BPP: u32 = 4;
const FB_STRIDE: u32 = FB_WIDTH * FB_BPP;
const FB_BYTES: usize = (FB_WIDTH * FB_HEIGHT * FB_BPP) as usize;

/// 8×8 source glyphs, painted with this integer scale for crisp pixels.
const FONT_SCALE: u32 = 2;
/// Horizontal advance: glyph width + 2px gap (avoids overlapping).
const CHAR_ADVANCE: u32 = 8 * FONT_SCALE + 2;

/// DRM_FORMAT_XRGB8888 = fourcc_code('X','R','2','4') = 0x34325258.
/// Stored big-endian in `RAMFBCfg` (QEMU `be32_to_cpu` on read).
const FOURCC_XR24: u32 = 0x3432_5258;

/// QEMU virt fw_cfg MMIO (within identity-mapped device window).
/// Layout: DATA @+0, SELECTOR @+8, DMA address @+16 (64-bit BE; write to +20 triggers).
const FW_CFG_BASE: usize = 0x0902_0000;
const FW_CFG_DATA: usize = FW_CFG_BASE;
const FW_CFG_SELECTOR: usize = FW_CFG_BASE + 0x08;
const FW_CFG_DMA_ADDR: usize = FW_CFG_BASE + 0x10;
const FW_CFG_FILE_DIR: u16 = 0x19;

const FW_CFG_DMA_CTL_ERROR: u32 = 0x01;
const FW_CFG_DMA_CTL_SELECT: u32 = 0x08;
const FW_CFG_DMA_CTL_WRITE: u32 = 0x10;

/// Guest-side DMA descriptor (all fields big-endian), per QEMU fw_cfg spec.
#[repr(C, packed)]
struct FwCfgDmaAccess {
    control: u32,
    length: u32,
    address: u64,
}

const VIRTIO_ID_GPU: u32 = 16;

pub fn init() {
    probe_virtio_gpu();
    match setup_ramfb() {
        Ok(_addr) => {
            console::println("display: ramfb smoke ok (solid fill + glyphs)");
        }
        Err(msg) => {
            console::print("display: ramfb skip - ");
            console::println(msg);
        }
    }
}

fn probe_virtio_gpu() {
    match virtio::find_device(VIRTIO_ID_GPU) {
        Some((base, version)) => {
            console::print("display: virtio-gpu at mmio ");
            print_hex_usize(base);
            console::print(" version=");
            console::print(match version {
                1 => "1 (legacy)",
                2 => "2 (modern)",
                _ => "?",
            });
            console::println("");
            match virtio::init_gpu_scanout() {
                Ok(()) => console::println(
                    "display: virtio-gpu scanout ok (checkerboard fill; ramfb still fallback)",
                ),
                Err(msg) => {
                    console::print("display: virtio-gpu scanout failed - ");
                    console::println(msg);
                    console::println("display: ramfb remains fallback");
                }
            }
        }
        None => console::println("display: no virtio-gpu device"),
    }
}

fn setup_ramfb() -> Result<usize, &'static str> {
    let select = find_fw_cfg_file(b"etc/ramfb").ok_or("no etc/ramfb (use run-qemu-gui.ps1)")?;
    let pages = (FB_BYTES + frame::PAGE_SIZE - 1) / frame::PAGE_SIZE;
    let addr = alloc_contig_pages(pages).ok_or("framebuffer alloc failed")?;

    // RAMFBCfg (28 bytes, all fields big-endian). Must live in guest RAM for DMA.
    let mut cfg = [0u8; 28];
    write_be64(&mut cfg[0..8], addr as u64);
    write_be32(&mut cfg[8..12], FOURCC_XR24);
    write_be32(&mut cfg[12..16], 0); // flags
    write_be32(&mut cfg[16..20], FB_WIDTH);
    write_be32(&mut cfg[20..24], FB_HEIGHT);
    write_be32(&mut cfg[24..28], FB_STRIDE);

    // Paint before activating scanout so the first host blit is not blank.
    smoke_draw(addr);

    fw_cfg_dma_write(select, &cfg)?;

    console::print("display: ramfb mapped ");
    print_u32(FB_WIDTH);
    console::print("x");
    print_u32(FB_HEIGHT);
    console::print(" @ ");
    print_hex_usize(addr);
    console::println(" (fw_cfg DMA)");
    Ok(addr)
}

fn smoke_draw(base: usize) {
    // XRGB8888 colors (host framebuffer atmosphere cue).
    let fill = 0x0018_5a6eu32;
    unsafe {
        let px = base as *mut u32;
        let n = (FB_WIDTH * FB_HEIGHT) as usize;
        for i in 0..n {
            core::ptr::write_volatile(px.add(i), fill);
        }
        // Brand header bar
        fill_rect(base, 0, 0, FB_WIDTH, 72, 0x000c_5a6e);
        draw_text(base, 24, 28, b"AURAOS", 0x00f0_faff);
        draw_text(base, 24, 120, b"HOME", 0x00e6_f0f5);
        draw_text(base, 24, 150, b"AGENT ALWAYS ON", 0x00b4_d2dc);
        // Agent panel (bottom third)
        fill_rect(base, 16, FB_HEIGHT - 280, FB_WIDTH - 32, 248, 0x0012_1c24);
        draw_text(base, 32, FB_HEIGHT - 250, b"AGENT CORE", 0x0078_dcc8);
        draw_text(base, 32, FB_HEIGHT - 210, b"ASK ANYTHING...", 0x00a0_aab4);
        draw_text(base, 32, FB_HEIGHT - 160, b"[HELP] [STATUS]", 0x0064_b4a0);
    }
}

unsafe fn fill_rect(base: usize, x0: u32, y0: u32, w: u32, h: u32, color: u32) {
    let px = base as *mut u32;
    let x1 = (x0 + w).min(FB_WIDTH);
    let y1 = (y0 + h).min(FB_HEIGHT);
    let mut y = y0;
    while y < y1 {
        let mut x = x0;
        while x < x1 {
            core::ptr::write_volatile(px.add((y * FB_WIDTH + x) as usize), color);
            x += 1;
        }
        y += 1;
    }
}

unsafe fn draw_text(base: usize, x: u32, y: u32, text: &[u8], color: u32) {
    let mut cx = x;
    for &ch in text {
        draw_char(base, cx, y, ch, color);
        cx += CHAR_ADVANCE;
        if cx + 8 * FONT_SCALE >= FB_WIDTH {
            break;
        }
    }
}

unsafe fn draw_char(base: usize, x: u32, y: u32, ch: u8, color: u32) {
    let glyph = glyph_for(ch);
    let px = base as *mut u32;
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..8u32 {
            if bits & (1 << (7 - col)) != 0 {
                let mut dy = 0u32;
                while dy < FONT_SCALE {
                    let mut dx = 0u32;
                    while dx < FONT_SCALE {
                        let sx = x + col * FONT_SCALE + dx;
                        let sy = y + (row as u32) * FONT_SCALE + dy;
                        if sx < FB_WIDTH && sy < FB_HEIGHT {
                            core::ptr::write_volatile(
                                px.add((sy * FB_WIDTH + sx) as usize),
                                color,
                            );
                        }
                        dx += 1;
                    }
                    dy += 1;
                }
            }
        }
    }
}

/// Clean 8x8 bitmap glyphs (MSB = leftmost pixel). Covers A-Z, 0-9, and smoke
/// punctuation; unknown chars render as a hollow box.
fn glyph_for(ch: u8) -> [u8; 8] {
    match ch.to_ascii_uppercase() {
        b' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        b'.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
        b'-' => [0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00],
        b'[' => [0x3C, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3C, 0x00],
        b']' => [0x3C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x3C, 0x00],
        b'0' => [0x3C, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x3C, 0x00],
        b'1' => [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7E, 0x00],
        b'2' => [0x3C, 0x66, 0x06, 0x0C, 0x18, 0x30, 0x7E, 0x00],
        b'3' => [0x3C, 0x66, 0x06, 0x1C, 0x06, 0x66, 0x3C, 0x00],
        b'4' => [0x0C, 0x1C, 0x3C, 0x6C, 0x7E, 0x0C, 0x0C, 0x00],
        b'5' => [0x7E, 0x60, 0x7C, 0x06, 0x06, 0x66, 0x3C, 0x00],
        b'6' => [0x1C, 0x30, 0x60, 0x7C, 0x66, 0x66, 0x3C, 0x00],
        b'7' => [0x7E, 0x06, 0x0C, 0x18, 0x30, 0x30, 0x30, 0x00],
        b'8' => [0x3C, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x3C, 0x00],
        b'9' => [0x3C, 0x66, 0x66, 0x3E, 0x06, 0x0C, 0x38, 0x00],
        b'A' => [0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00],
        b'B' => [0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x7C, 0x00],
        b'C' => [0x3C, 0x66, 0x60, 0x60, 0x60, 0x66, 0x3C, 0x00],
        b'D' => [0x78, 0x6C, 0x66, 0x66, 0x66, 0x6C, 0x78, 0x00],
        b'E' => [0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x7E, 0x00],
        b'F' => [0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x60, 0x00],
        b'G' => [0x3C, 0x66, 0x60, 0x6E, 0x66, 0x66, 0x3C, 0x00],
        b'H' => [0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x00],
        b'I' => [0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0x00],
        b'J' => [0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x6C, 0x38, 0x00],
        b'K' => [0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x00],
        b'L' => [0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0x00],
        b'M' => [0x63, 0x77, 0x7F, 0x6B, 0x63, 0x63, 0x63, 0x00],
        b'N' => [0x66, 0x76, 0x7E, 0x7E, 0x6E, 0x66, 0x66, 0x00],
        b'O' => [0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
        b'P' => [0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x00],
        b'Q' => [0x3C, 0x66, 0x66, 0x66, 0x6A, 0x6C, 0x36, 0x00],
        b'R' => [0x7C, 0x66, 0x66, 0x7C, 0x78, 0x6C, 0x66, 0x00],
        b'S' => [0x3C, 0x66, 0x60, 0x3C, 0x06, 0x66, 0x3C, 0x00],
        b'T' => [0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
        b'U' => [0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x00],
        b'V' => [0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x00],
        b'W' => [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00],
        b'X' => [0x66, 0x66, 0x3C, 0x18, 0x3C, 0x66, 0x66, 0x00],
        b'Y' => [0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0x18, 0x00],
        b'Z' => [0x7E, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x7E, 0x00],
        _ => [0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x00],
    }
}

fn alloc_contig_pages(pages: usize) -> Option<usize> {
    let first = frame::alloc_frame()?;
    for i in 1..pages {
        let p = frame::alloc_frame()?;
        if p != first + i * frame::PAGE_SIZE {
            return None;
        }
    }
    Some(first)
}

fn find_fw_cfg_file(name: &[u8]) -> Option<u16> {
    fw_cfg_select(FW_CFG_FILE_DIR);
    let count = fw_cfg_read_be32();
    if count == 0 || count > 64 {
        return None;
    }
    for _ in 0..count {
        let _size = fw_cfg_read_be32();
        let select = fw_cfg_read_be16();
        let _reserved = fw_cfg_read_be16();
        let mut fname = [0u8; 56];
        for b in &mut fname {
            *b = fw_cfg_read8();
        }
        let end = fname.iter().position(|&c| c == 0).unwrap_or(56);
        if &fname[..end] == name {
            return Some(select);
        }
    }
    None
}

fn fw_cfg_select(key: u16) {
    unsafe {
        // Selector is big-endian on MMIO.
        let be = key.to_be();
        core::ptr::write_volatile(FW_CFG_SELECTOR as *mut u16, be);
    }
}

fn fw_cfg_read8() -> u8 {
    unsafe { core::ptr::read_volatile(FW_CFG_DATA as *const u8) }
}

fn fw_cfg_read_be16() -> u16 {
    let b0 = fw_cfg_read8() as u16;
    let b1 = fw_cfg_read8() as u16;
    (b0 << 8) | b1
}

fn fw_cfg_read_be32() -> u32 {
    let b0 = fw_cfg_read8() as u32;
    let b1 = fw_cfg_read8() as u32;
    let b2 = fw_cfg_read8() as u32;
    let b3 = fw_cfg_read8() as u32;
    (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
}

/// Write `buf` into the selected fw_cfg file via the DMA interface.
///
/// Ramfb's write callback (`ramfb_fw_cfg_write`) runs when this completes; the
/// classic DATA-register store path does not perform guest→host writes.
fn fw_cfg_dma_write(select: u16, buf: &[u8]) -> Result<(), &'static str> {
    let mut access = FwCfgDmaAccess {
        control: 0,
        length: 0,
        address: 0,
    };
    let control =
        ((select as u32) << 16) | FW_CFG_DMA_CTL_SELECT | FW_CFG_DMA_CTL_WRITE;
    access.control = control.to_be();
    access.length = (buf.len() as u32).to_be();
    access.address = (buf.as_ptr() as u64).to_be();

    // Ensure DMA descriptor + payload are visible to the host before trigger.
    dsb_sy();

    let access_phys = core::ptr::from_ref(&access) as u64;
    // DMA address register is big-endian; writing the low half (offset +4) starts
    // the transfer. Clear high half first (addresses are below 4G in virt RAM).
    unsafe {
        core::ptr::write_volatile(FW_CFG_DMA_ADDR as *mut u32, 0u32);
        core::ptr::write_volatile(
            (FW_CFG_DMA_ADDR + 4) as *mut u32,
            (access_phys as u32).to_be(),
        );
    }

    dsb_sy();

    // QEMU completes DMA synchronously; control clears to 0 on success.
    let mut spins = 0u32;
    loop {
        let ctl = u32::from_be(unsafe {
            core::ptr::read_volatile(core::ptr::addr_of!(access.control))
        });
        if ctl & FW_CFG_DMA_CTL_ERROR != 0 {
            return Err("fw_cfg DMA write error");
        }
        if ctl == 0 {
            return Ok(());
        }
        spins = spins.wrapping_add(1);
        if spins > 1_000_000 {
            return Err("fw_cfg DMA write timeout");
        }
    }
}

#[inline(always)]
fn dsb_sy() {
    unsafe {
        core::arch::asm!("dsb sy", options(nostack, preserves_flags));
    }
}

fn write_be32(dst: &mut [u8], v: u32) {
    dst[0] = (v >> 24) as u8;
    dst[1] = (v >> 16) as u8;
    dst[2] = (v >> 8) as u8;
    dst[3] = v as u8;
}

fn write_be64(dst: &mut [u8], v: u64) {
    for i in 0..8 {
        dst[i] = (v >> (56 - i * 8)) as u8;
    }
}

fn print_hex_usize(v: usize) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    console::print("0x");
    let mut started = false;
    for i in (0..16).rev() {
        let nibble = ((v >> (i * 4)) & 0xf) as usize;
        if nibble != 0 || started || i == 0 {
            started = true;
            let b = [HEX[nibble]];
            crate::uart::write_bytes(&b);
        }
    }
}

fn print_u32(mut v: u32) {
    if v == 0 {
        console::print("0");
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 10;
    while v > 0 {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    crate::uart::write_bytes(&buf[i..]);
}
