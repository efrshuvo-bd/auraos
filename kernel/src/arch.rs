//! Architecture helpers for aarch64 QEMU virt.

use core::arch::{asm, naked_asm};

const STACK_SIZE: usize = 65_536;

#[repr(C, align(16))]
struct KernelStack([u8; STACK_SIZE]);

#[used]
static mut KERNEL_STACK: KernelStack = KernelStack([0; STACK_SIZE]);

/// QEMU `-kernel` passes the FDT pointer in `x0`. Preserve it into `kernel_main(x0)`.
#[unsafe(naked)]
#[no_mangle]
#[link_section = ".text.boot"]
pub unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        // x0 = FDT from QEMU; keep in x19 across stack setup.
        "mov x19, x0",
        "adrp x0, {stack}",
        "add x0, x0, :lo12:{stack}",
        "mov x1, {size}",
        "add x0, x0, x1",
        "mov sp, x0",
        "mov x0, x19",
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

/// Allow NEON/FP at EL1 (target enables `+neon,+fp-armv8`; compiler may emit `movi` etc.).
pub fn enable_fp_simd() {
    unsafe {
        let mut cpacr: u64;
        asm!("mrs {0}, cpacr_el1", out(reg) cpacr, options(nostack));
        cpacr |= 0b11 << 20; // FPEN = no trap at EL0/EL1
        asm!("msr cpacr_el1, {0}", in(reg) cpacr, options(nostack));
        asm!("isb", options(nostack));
    }
}

pub fn current_el() -> u64 {
    let el: u64;
    unsafe {
        asm!("mrs {0}, CurrentEL", out(reg) el, options(nomem, nostack));
    }
    (el >> 2) & 0b11
}
