//! ARM generic physical timer (CNTP) — preemptive ticks via GICv2 PPI 30.

use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::console;
use crate::gic;

static TICKS: AtomicU64 = AtomicU64::new(0);
static ARMED: AtomicBool = AtomicBool::new(false);
static mut INTERVAL: u32 = 0;

/// ~10 ms tick (derived from CNTFRQ_EL0).
const TICK_HZ: u32 = 100;

pub fn init() {
    TICKS.store(0, Ordering::SeqCst);
    gic::init();

    let freq = cntfrq();
    if freq == 0 {
        console::println("timer: CNTFRQ=0; IRQ timer disabled");
        return;
    }
    let interval = (freq / TICK_HZ).max(1000);
    unsafe {
        INTERVAL = interval;
    }

    // Program non-secure physical timer.
    set_tval(interval);
    // ENABLE=1, IMASK=0
    unsafe {
        asm!("msr cntp_ctl_el0, {0}", in(reg) 1u64, options(nomem, nostack));
        asm!("isb", options(nomem, nostack));
    }
    ARMED.store(true, Ordering::SeqCst);
    console::println("timer: CNTP armed (100 Hz, IRQ preempt)");
}

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub fn is_armed() -> bool {
    ARMED.load(Ordering::Acquire)
}

/// Soft tick (cooperative yield path).
pub fn tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

/// Handle a timer IRQ: bump tick, re-arm. Returns true if this was CNTP.
pub fn handle_irq(irq: u32) -> bool {
    if irq != gic::IRQ_CNTP {
        return false;
    }
    TICKS.fetch_add(1, Ordering::Relaxed);
    rearm();
    true
}

fn rearm() {
    let interval = unsafe { INTERVAL };
    if interval != 0 {
        set_tval(interval);
    }
}

fn set_tval(v: u32) {
    unsafe {
        asm!("msr cntp_tval_el0, {0}", in(reg) v as u64, options(nomem, nostack));
        asm!("isb", options(nomem, nostack));
    }
}

fn cntfrq() -> u32 {
    let v: u64;
    unsafe {
        asm!("mrs {0}, cntfrq_el0", out(reg) v, options(nomem, nostack));
    }
    v as u32
}
