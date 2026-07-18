//! Preemptive round-robin scheduler for EL0 processes.
//!
//! Yield, Exit, and Timer Preempt all return through `process::return_to_kernel`
//! → `bridge_from_el0` → `sched::run`.

use crate::console;
use crate::process;
use crate::timer;
use crate::trap::TrapAction;
use crate::virtio;
use core::arch::asm;
use core::sync::atomic::{AtomicUsize, Ordering};

static NEXT_IDX: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    NEXT_IDX.store(0, Ordering::SeqCst);
}

pub fn yield_now() {
    timer::tick();
}

/// Run until no Runnable processes remain.
/// Safe to re-enter from the EL0 bridge stack after a trap/IRQ.
pub fn run() -> ! {
    loop {
        let slots = process::slot_count();
        if slots == 0 {
            console::println("sched: no tasks; halting");
            idle_loop();
        }

        let mut ran_any = false;
        let start = NEXT_IDX.load(Ordering::Relaxed) % slots.max(1);
        for off in 0..slots {
            let i = (start + off) % slots;
            if !process::is_runnable(i) {
                continue;
            }
            ran_any = true;
            // Skip "run" spam when continuing the same task after a timer preempt.
            if process::last_action() != 3 {
                console::print("sched: run ");
                console::println(process::name_at(i));
            }
            NEXT_IDX.store(i + 1, Ordering::Relaxed);
            let _action: TrapAction = process::run(i);
            // Unreachable on success — bridge re-enters sched::run().
        }

        if !ran_any {
            console::println("sched: idle");
            idle_loop();
        }
    }
}

fn idle_loop() -> ! {
    loop {
        virtio::poll();
        // Unmask IRQs so CNTP can wake WFI; EL1 IRQ handler re-arms the timer.
        unsafe {
            asm!("msr daifclr, #0x2", options(nomem, nostack));
        }
        crate::arch::wait_for_interrupt();
        unsafe {
            asm!("msr daifset, #0x2", options(nomem, nostack));
        }
    }
}
