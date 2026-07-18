//! Architecture helpers for aarch64 QEMU virt.

use core::arch::{asm, naked_asm};

const STACK_SIZE: usize = 65_536;

#[repr(C, align(16))]
struct KernelStack([u8; STACK_SIZE]);

#[used]
static mut KERNEL_STACK: KernelStack = KernelStack([0; STACK_SIZE]);

#[unsafe(naked)]
#[no_mangle]
#[link_section = ".text.boot"]
pub unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        "adrp x0, {stack}",
        "add x0, x0, :lo12:{stack}",
        "mov x1, {size}",
        "add x0, x0, x1",
        "mov sp, x0",
        "b {main}",
        stack = sym KERNEL_STACK,
        size = const STACK_SIZE,
        main = sym crate::kernel_main,
    )
}

#[inline(always)]
pub fn wait_for_interrupt() {
    unsafe { asm!("wfi", options(nomem, nostack)) }
}

pub fn current_el() -> u64 {
    let el: u64;
    unsafe {
        asm!("mrs {0}, CurrentEL", out(reg) el, options(nomem, nostack));
    }
    (el >> 2) & 0b11
}
