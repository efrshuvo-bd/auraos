//! Syscall numbers and dispatch for AuraOS.

use crate::console;
use crate::ipc;
use crate::process;
use crate::sched;
use crate::trap::TrapFrame;
use crate::uart;
use crate::userspace;
use crate::virtio;

pub const SYS_WRITE: u64 = 1;
pub const SYS_YIELD: u64 = 2;
pub const SYS_EXIT: u64 = 3;
pub const SYS_IPC_SEND: u64 = 4;
pub const SYS_IPC_RECV: u64 = 5;
pub const SYS_READ: u64 = 6;
/// Waitpid: a0 = pid (0 or -1 = any). Returns packed `(pid<<32)|status` or -1.
pub const SYS_WAITPID: u64 = 7;
/// Init-only spawn from initrd: a0 = role (1=agent, 2=shell). Returns pid or -1.
pub const SYS_SPAWN: u64 = 8;

const USER_VA_MAX: u64 = 0x0000_0000_0080_0000;

pub fn init() {
    console::println("syscall: table ready (write/read/yield/exit/ipc/waitpid/spawn)");
}

/// Trap-based syscall entry (x8=nr, args in x0..).
pub fn dispatch_trap(num: u64, tf: &mut TrapFrame) -> i64 {
    let a0 = tf.x[0];
    let a1 = tf.x[1];
    match num {
        SYS_WRITE => sys_write(a0, a1),
        SYS_READ => sys_read(a0, a1),
        SYS_YIELD => {
            sched::yield_now();
            0
        }
        SYS_EXIT => {
            process::set_exit_status(a0 as i32);
            console::println("syscall: exit");
            0
        }
        SYS_IPC_SEND => {
            if ipc::send(a0 as u32, a1) {
                0
            } else {
                -1
            }
        }
        SYS_IPC_RECV => ipc::recv(a0 as u32) as i64,
        SYS_WAITPID => sys_waitpid(a0),
        SYS_SPAWN => userspace::spawn_from_initrd(a0),
        _ => {
            console::println("syscall: unknown");
            -1
        }
    }
}

/// Non-blocking waitpid. Success packs reaped pid (high 32) and status (low 32).
fn sys_waitpid(pid: u64) -> i64 {
    let waiter = process::current_pid();
    // Accept 0 or -1 (all-bits-set) as wait-any (SCRUM-39).
    let target = pid as u32;
    match process::waitpid_noblock(waiter, target) {
        Some((reaped_pid, status)) => {
            ((reaped_pid as i64) << 32) | ((status as u32) as i64)
        }
        None => -1,
    }
}

fn sys_write(ptr: u64, len: u64) -> i64 {
    if len == 0 {
        return 0;
    }
    if len > 4096 || ptr == 0 || ptr.saturating_add(len) > USER_VA_MAX {
        return -1;
    }
    let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
    // Prefer VirtIO console TX; fall back to UART if VirtIO is absent/failed.
    if !virtio::write_bytes(slice) {
        uart::write_bytes(slice);
    }
    len as i64
}

/// Non-blocking read from VirtIO console RX (returns 0 when empty).
fn sys_read(ptr: u64, len: u64) -> i64 {
    if len == 0 {
        return 0;
    }
    if len > 4096 || ptr == 0 || ptr.saturating_add(len) > USER_VA_MAX {
        return -1;
    }
    let slice = unsafe { core::slice::from_raw_parts_mut(ptr as *mut u8, len as usize) };
    match virtio::read_bytes(slice) {
        Some(n) => n as i64,
        None => -1,
    }
}
