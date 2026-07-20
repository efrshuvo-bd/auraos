//! Physical frame allocator over a reserved RAM window.
//!
//! Sprint 9: bump alloc + freelist so process Exit can return TTBR0 / user pages.

use core::sync::atomic::{AtomicUsize, Ordering};

pub const PAGE_SIZE: usize = 4096;

/// Max freelist entries (enough for a few process address spaces).
const FREE_CAP: usize = 512;

static FRAME_BASE: AtomicUsize = AtomicUsize::new(0);
static FRAME_END: AtomicUsize = AtomicUsize::new(0);
static FRAME_NEXT: AtomicUsize = AtomicUsize::new(0);
static FRAMES_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static FRAMES_FREED: AtomicUsize = AtomicUsize::new(0);

static mut FREE_LIST: [usize; FREE_CAP] = [0; FREE_CAP];
static FREE_LEN: AtomicUsize = AtomicUsize::new(0);

pub fn init(base: usize, size: usize) {
    let end = base + size;
    FRAME_BASE.store(base, Ordering::SeqCst);
    FRAME_END.store(end, Ordering::SeqCst);
    FRAME_NEXT.store(base, Ordering::SeqCst);
    FRAMES_ALLOCATED.store(0, Ordering::SeqCst);
    FRAMES_FREED.store(0, Ordering::SeqCst);
    FREE_LEN.store(0, Ordering::SeqCst);
}

pub fn alloc_frame() -> Option<usize> {
    // Prefer freelist so Exit can recycle TTBR0 / user pages.
    loop {
        let len = FREE_LEN.load(Ordering::Relaxed);
        if len == 0 {
            break;
        }
        if FREE_LEN
            .compare_exchange(len, len - 1, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            let phys = unsafe { FREE_LIST[len - 1] };
            FRAMES_ALLOCATED.fetch_add(1, Ordering::Relaxed);
            zero_page(phys);
            return Some(phys);
        }
    }

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

/// Return a frame to the freelist (SCRUM-47). Drops if freelist is full.
pub fn free_frame(phys: usize) {
    let base = FRAME_BASE.load(Ordering::Relaxed);
    let end = FRAME_END.load(Ordering::Relaxed);
    if phys < base || phys >= end || (phys % PAGE_SIZE) != 0 {
        return;
    }
    loop {
        let len = FREE_LEN.load(Ordering::Relaxed);
        if len >= FREE_CAP {
            return;
        }
        if FREE_LEN
            .compare_exchange(len, len + 1, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            unsafe {
                FREE_LIST[len] = phys;
            }
            FRAMES_FREED.fetch_add(1, Ordering::Relaxed);
            return;
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

pub fn frames_freed() -> usize {
    FRAMES_FREED.load(Ordering::Relaxed)
}

pub fn page_size() -> usize {
    PAGE_SIZE
}
