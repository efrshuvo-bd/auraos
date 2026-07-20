//! Tiny EL0 syscall helpers for AuraOS guest programs.

#![no_std]

pub mod agent_ipc;

pub const SYS_WRITE: u64 = 1;
pub const SYS_YIELD: u64 = 2;
pub const SYS_EXIT: u64 = 3;
pub const SYS_IPC_SEND: u64 = 4;
pub const SYS_IPC_RECV: u64 = 5;
pub const SYS_READ: u64 = 6;
/// Non-blocking waitpid (pid=0 → any exited peer). Returns status or -1.
pub const SYS_WAITPID: u64 = 7;

#[inline(always)]
pub unsafe fn syscall3(nr: u64, a0: u64, a1: u64, a2: u64) -> i64 {
    let ret: i64;
    core::arch::asm!(
        "mov x8, {nr}",
        "mov x0, {a0}",
        "mov x1, {a1}",
        "mov x2, {a2}",
        "svc #0",
        "mov {ret}, x0",
        nr = in(reg) nr,
        a0 = in(reg) a0,
        a1 = in(reg) a1,
        a2 = in(reg) a2,
        ret = lateout(reg) ret,
        out("x8") _,
        out("x0") _,
        out("x1") _,
        out("x2") _,
        options(nostack),
    );
    ret
}

pub fn write(s: &str) {
    unsafe {
        let _ = syscall3(SYS_WRITE, s.as_ptr() as u64, s.len() as u64, 0);
    }
}

/// Non-blocking console read via VirtIO RX. Returns bytes copied (0 if empty).
pub fn read(buf: &mut [u8]) -> usize {
    let n = unsafe { syscall3(SYS_READ, buf.as_mut_ptr() as u64, buf.len() as u64, 0) };
    if n < 0 {
        0
    } else {
        n as usize
    }
}

pub fn yield_now() {
    unsafe {
        let _ = syscall3(SYS_YIELD, 0, 0, 0);
    }
}

pub fn exit() -> ! {
    exit_with(0)
}

pub fn exit_with(status: i32) -> ! {
    unsafe {
        let _ = syscall3(SYS_EXIT, status as u64, 0, 0);
    }
    loop {}
}

/// Non-blocking waitpid. `pid == 0` waits for any exited peer process.
/// Returns `Some(status)` if a process was reaped, `None` if none ready.
pub fn waitpid_noblock(pid: u32) -> Option<i32> {
    let n = unsafe { syscall3(SYS_WAITPID, pid as u64, 0, 0) };
    if n < 0 {
        None
    } else {
        Some(n as i32)
    }
}

pub fn ipc_send(channel: u64, payload: u64) {
    unsafe {
        let _ = syscall3(SYS_IPC_SEND, channel, payload, 0);
    }
}

pub fn ipc_recv(channel: u64) -> u64 {
    unsafe { syscall3(SYS_IPC_RECV, channel, 0, 0) as u64 }
}

/// Poll mailbox until non-zero or `max_yields` exhausted. Returns 0 on timeout.
pub fn ipc_recv_wait(channel: u64, max_yields: u32) -> u64 {
    for _ in 0..max_yields {
        let v = ipc_recv(channel);
        if v != 0 {
            return v;
        }
        yield_now();
    }
    0
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    write("guest panic\n");
    exit()
}
