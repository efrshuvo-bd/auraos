//! AuraOS kernel — QEMU virt aarch64.

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

mod arch;
mod board_pi5;
mod console;
mod frame;
mod ipc;
mod mem;
mod sched;
mod syscall;
mod timer;
mod uart;
mod userspace;
mod vm;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    let _board = board_pi5::BOARD;
    uart::init();
    console::println("AuraOS kernel online");
    console::println("phase1: uart + panic + frame allocator");

    mem::init_heap();
    frame::init(0x4400_0000, 64 * 1024 * 1024);
    console::println("phase2: heap + frame allocator ready");

    vm::init_identity_map();
    console::println("phase2: identity map installed (stub)");

    timer::init();
    console::println("phase2: timer tick armed");

    sched::init();
    syscall::init();
    ipc::init();
    console::println("phase2: scheduler + syscalls + ipc ready");

    userspace::spawn_init();
    console::println("phase2/3: userspace init scheduled");

    console::println("AuraOS entering scheduler");
    sched::run()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    console::print("KERNEL PANIC: ");
    if let Some(loc) = info.location() {
        console::print(loc.file());
        console::print(":");
        // Avoid format machinery for location line in no_std minimally:
        console::println("");
    } else {
        console::println("unknown location");
    }
    loop {
        arch::wait_for_interrupt();
    }
}

#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    console::println("alloc error");
    loop {
        arch::wait_for_interrupt();
    }
}
