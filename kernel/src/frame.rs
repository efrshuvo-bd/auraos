//! Physical frame allocator over a reserved RAM window.

use core::sync::atomic::{AtomicUsize, Ordering};

pub const PAGE_SIZE: usize = 4096;

static FRAME_BASE: AtomicUsize = AtomicUsize::new(0);
static FRAME_END: AtomicUsize = AtomicUsize::new(0);
static FRAME_NEXT: AtomicUsize = AtomicUsize::new(0);
static FRAMES_ALLOCATED: AtomicUsize = AtomicUsize::new(0);

pub fn init(base: usize, size: usize) {
    let end = base + size;
    FRAME_BASE.store(base, Ordering::SeqCst);
    FRAME_END.store(end, Ordering::SeqCst);
    FRAME_NEXT.store(base, Ordering::SeqCst);
    FRAMES_ALLOCATED.store(0, Ordering::SeqCst);
}

pub fn alloc_frame() -> Option<usize> {
    loop {
        let cur = FRAME_NEXT.load(Ordering::Relaxed);
        let end = FRAME_END.load(Ordering::Relaxed);
        let next = cur.checked_add(PAGE_SIZE)?;
        if next > end {
            return None;
        }
        if FRAME_NEXT
            .compare_exchange(cur, next, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            FRAMES_ALLOCATED.fetch_add(1, Ordering::Relaxed);
            zero_page(cur);
            return Some(cur);
        }
    }
}

pub fn zero_page(phys: usize) {
    unsafe {
        core::ptr::write_bytes(phys as *mut u8, 0, PAGE_SIZE);
    }
}

pub fn frames_allocated() -> usize {
    FRAMES_ALLOCATED.load(Ordering::Relaxed)
}

pub fn page_size() -> usize {
    PAGE_SIZE
}
