//! Generic timer stub for aarch64.

use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    // Full CNTP programming requires EL config; count soft ticks from scheduler.
    TICKS.store(0, Ordering::SeqCst);
}

pub fn tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}
