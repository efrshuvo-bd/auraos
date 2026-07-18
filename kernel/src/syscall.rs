//! Syscall numbers and dispatch for AuraOS.

use crate::console;
use crate::ipc;
use crate::sched;

pub const SYS_WRITE: u64 = 1;
pub const SYS_YIELD: u64 = 2;
pub const SYS_EXIT: u64 = 3;
pub const SYS_IPC_SEND: u64 = 4;
pub const SYS_IPC_RECV: u64 = 5;

pub fn init() {
    console::println("syscall: table ready (write/yield/exit/ipc)");
}

/// Kernel-side syscall entry used by simulated userspace tasks.
pub fn dispatch(num: u64, a0: u64, a1: u64, _a2: u64) -> i64 {
    match num {
        SYS_WRITE => {
            // a0 = ptr, a1 = len — for simulated userspace we accept a static message id
            let _ = (a0, a1);
            console::println("syscall: write");
            0
        }
        SYS_YIELD => {
            sched::yield_now();
            0
        }
        SYS_EXIT => {
            console::println("syscall: exit");
            0
        }
        SYS_IPC_SEND => {
            let ok = ipc::send(a0 as u32, a1);
            if ok {
                0
            } else {
                -1
            }
        }
        SYS_IPC_RECV => ipc::recv(a0 as u32) as i64,
        _ => {
            console::println("syscall: unknown");
            -1
        }
    }
}
