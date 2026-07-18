//! Cooperative scheduler for AuraOS kernel tasks / userspace fibers.

use crate::console;
use crate::timer;
use core::sync::atomic::{AtomicUsize, Ordering};

pub type TaskFn = fn();

const MAX_TASKS: usize = 8;

#[derive(Copy, Clone)]
struct Task {
    name: &'static str,
    entry: Option<TaskFn>,
    alive: bool,
}

static mut TASKS: [Task; MAX_TASKS] = [Task {
    name: "",
    entry: None,
    alive: false,
}; MAX_TASKS];

static TASK_COUNT: AtomicUsize = AtomicUsize::new(0);
static CURRENT: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    TASK_COUNT.store(0, Ordering::SeqCst);
    CURRENT.store(0, Ordering::SeqCst);
}

pub fn spawn(name: &'static str, entry: TaskFn) -> bool {
    let idx = TASK_COUNT.load(Ordering::Relaxed);
    if idx >= MAX_TASKS {
        return false;
    }
    unsafe {
        TASKS[idx] = Task {
            name,
            entry: Some(entry),
            alive: true,
        };
    }
    TASK_COUNT.fetch_add(1, Ordering::SeqCst);
    true
}

pub fn yield_now() {
    timer::tick();
}

/// Run tasks round-robin until none remain alive.
pub fn run() -> ! {
    loop {
        let count = TASK_COUNT.load(Ordering::Relaxed);
        if count == 0 {
            console::println("sched: no tasks; halting");
            crate::arch::wait_for_interrupt();
            continue;
        }
        let mut ran_any = false;
        for i in 0..count {
            CURRENT.store(i, Ordering::Relaxed);
            let (alive, entry, name) = unsafe {
                let t = &TASKS[i];
                (t.alive, t.entry, t.name)
            };
            if !alive {
                continue;
            }
            ran_any = true;
            if let Some(f) = entry {
                console::print("sched: run ");
                console::println(name);
                f();
                // Cooperative: one-shot tasks complete and die.
                unsafe {
                    TASKS[i].alive = false;
                }
            }
            yield_now();
        }
        if !ran_any {
            console::println("sched: idle");
            crate::arch::wait_for_interrupt();
        }
    }
}

pub fn current_name() -> &'static str {
    let i = CURRENT.load(Ordering::Relaxed);
    unsafe { TASKS[i].name }
}
