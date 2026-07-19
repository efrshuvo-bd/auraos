//! Early boot info captured from QEMU (`x0` = FDT pointer).

use core::sync::atomic::{AtomicUsize, Ordering};

static BOOT_FDT: AtomicUsize = AtomicUsize::new(0);
static INITRD_START: AtomicUsize = AtomicUsize::new(0);
static INITRD_END: AtomicUsize = AtomicUsize::new(0);

pub fn init(fdt: usize) {
    BOOT_FDT.store(fdt, Ordering::SeqCst);
    if fdt == 0 {
        return;
    }
    if let Some((start, end)) = crate::fdt::initrd_range(fdt) {
        if end > start {
            INITRD_START.store(start, Ordering::SeqCst);
            INITRD_END.store(end, Ordering::SeqCst);
        }
    }
}

pub fn fdt_ptr() -> usize {
    BOOT_FDT.load(Ordering::SeqCst)
}

pub fn initrd_slice() -> Option<&'static [u8]> {
    let start = INITRD_START.load(Ordering::SeqCst);
    let end = INITRD_END.load(Ordering::SeqCst);
    if start == 0 || end <= start {
        return None;
    }
    let len = end - start;
    Some(unsafe { core::slice::from_raw_parts(start as *const u8, len) })
}

pub fn initrd_range() -> Option<(usize, usize)> {
    let start = INITRD_START.load(Ordering::SeqCst);
    let end = INITRD_END.load(Ordering::SeqCst);
    if start == 0 || end <= start {
        None
    } else {
        Some((start, end))
    }
}
