//! Tiny EL0 syscall helpers for AuraOS guest programs.

#![no_std]

pub const SYS_WRITE: u64 = 1;
pub const SYS_YIELD: u64 = 2;
pub const SYS_EXIT: u64 = 3;
pub const SYS_IPC_SEND: u64 = 4;
pub const SYS_IPC_RECV: u64 = 5;

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

pub fn yield_now() {
    unsafe {
        let _ = syscall3(SYS_YIELD, 0, 0, 0);
    }
}

pub fn exit() -> ! {
    unsafe {
        let _ = syscall3(SYS_EXIT, 0, 0, 0);
    }
    loop {}
}

pub fn ipc_send(channel: u64, payload: u64) {
    unsafe {
        let _ = syscall3(SYS_IPC_SEND, channel, payload, 0);
    }
}

pub fn ipc_recv(channel: u64) -> u64 {
    unsafe { syscall3(SYS_IPC_RECV, channel, 0, 0) as u64 }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    write("guest panic\n");
    exit()
}
