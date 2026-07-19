//! AuraOS kernel — QEMU virt aarch64.

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

mod arch;
mod board_pi5;
mod bootinfo;
mod console;
mod cpio;
mod display;
mod elf;
mod exceptions;
mod fdt;
mod frame;
mod gic;
mod ipc;
mod mem;
mod ota;
mod process;
mod sched;
mod syscall;
mod timer;
mod trap;
mod uart;
mod userspace;
mod virtio;
mod vm;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn kernel_main(fdt: usize) -> ! {
    let _board = board_pi5::BOARD;
    uart::init();
    arch::enable_fp_simd();
    console::println("AuraOS kernel online");
    console::println(board_pi5::status_line());
    console::println("phase1: uart + panic + frame allocator");

    bootinfo::init(fdt);
    if fdt == 0 {
        console::println("boot: FDT pointer is null");
    } else if bootinfo::initrd_range().is_some() {
        console::println("boot: initrd range from FDT /chosen");
    } else {
        console::println("boot: FDT present but no linux,initrd-* in /chosen");
    }

    mem::init_heap();
    // Frame pool after kernel load area; QEMU places initrd near end of RAM.
    frame::init(0x4400_0000, 64 * 1024 * 1024);
    console::println("phase2: heap + frame allocator ready");

    vm::init_identity_map();
    console::println("phase2: identity map installed");

    exceptions::init();
    virtio::init();
    virtio::probe_block_stub();
    ota::init();
    display::init();
    timer::init();
    console::println("phase2: gic + timer IRQ armed");

    process::init();
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
        console::println("");
        let _ = loc;
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
