//! Cooperative scheduler for EL0 processes.

use crate::console;
use crate::process;
use crate::timer;
use crate::trap::TrapAction;
use crate::virtio;
use core::sync::atomic::{AtomicUsize, Ordering};

static NEXT_IDX: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    NEXT_IDX.store(0, Ordering::SeqCst);
}

pub fn yield_now() {
    timer::tick();
}

/// Run embedded EL0 processes until all have exited.
/// Safe to re-enter from the EL0 bridge stack after a trap.
pub fn run() -> ! {
    loop {
        let count = process::count();
        if count == 0 {
            console::println("sched: no tasks; halting");
            crate::arch::wait_for_interrupt();
            continue;
        }

        let mut ran_any = false;
        let start = NEXT_IDX.load(Ordering::Relaxed) % count.max(1);
        for off in 0..count {
            let i = (start + off) % count;
            if !process::is_alive(i) {
                continue;
            }
            ran_any = true;
            console::print("sched: run ");
            console::println(process::name_at(i));
            NEXT_IDX.store(i + 1, Ordering::Relaxed);
            // process::run erets to EL0; trap path re-enters sched::run().
            let _action: TrapAction = process::run(i);
            // Unreachable on success.
        }

        if !ran_any {
            console::println("sched: idle");
            loop {
                // Polled VirtIO RX drain (IRQ path deferred).
                virtio::poll();
                crate::arch::wait_for_interrupt();
            }
        }
    }
}
