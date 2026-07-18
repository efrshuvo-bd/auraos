//! Exception vectors: EL0 SVC + EL0/EL1 IRQ (timer preempt).

use crate::arch;
use crate::console;
use crate::gic;
use crate::process;
use crate::syscall;
use crate::timer;
use crate::trap::{TrapAction, TrapFrame};
use core::arch::{asm, naked_asm};

/// Exception vector table — each entry is 0x80 bytes (`.align 7`).
#[unsafe(naked)]
#[no_mangle]
#[link_section = ".text.vectors"]
pub unsafe extern "C" fn exception_vectors() {
    naked_asm!(
        // Current EL with SP_EL0
        "b {el1_unhandled}",
        ".align 7",
        "b {el1_unhandled}",
        ".align 7",
        "b {el1_unhandled}",
        ".align 7",
        "b {el1_unhandled}",
        ".align 7",
        // Current EL with SP_ELx
        "b {el1_unhandled}",
        ".align 7",
        "b {el1_irq}",
        ".align 7",
        "b {el1_unhandled}",
        ".align 7",
        "b {el1_unhandled}",
        ".align 7",
        // Lower EL using AArch64 — sync (SVC) / IRQ
        "b {el0_sync}",
        ".align 7",
        "b {el0_irq}",
        ".align 7",
        "b {el1_unhandled}",
        ".align 7",
        "b {el1_unhandled}",
        ".align 7",
        // Lower EL using AArch32
        "b {el1_unhandled}",
        ".align 7",
        "b {el1_unhandled}",
        ".align 7",
        "b {el1_unhandled}",
        ".align 7",
        "b {el1_unhandled}",
        el1_unhandled = sym el1_unhandled,
        el1_irq = sym el1_irq_entry,
        el0_sync = sym el0_sync_entry,
        el0_irq = sym el0_irq_entry,
    )
}

#[unsafe(naked)]
unsafe extern "C" fn el1_unhandled() -> ! {
    naked_asm!(
        "b {handler}",
        handler = sym el1_unhandled_rust,
    )
}

fn el1_unhandled_rust() -> ! {
    let esr: u64;
    let elr: u64;
    let far: u64;
    unsafe {
        asm!("mrs {0}, esr_el1", out(reg) esr, options(nostack));
        asm!("mrs {0}, elr_el1", out(reg) elr, options(nostack));
        asm!("mrs {0}, far_el1", out(reg) far, options(nostack));
    }
    console::println("exception: unexpected EL1 fault");
    console::print("  esr=");
    print_hex(esr);
    console::print(" elr=");
    print_hex(elr);
    console::print(" far=");
    print_hex(far);
    console::println("");
    loop {
        arch::wait_for_interrupt();
    }
}

fn print_hex(v: u64) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    console::print("0x");
    for i in (0..16).rev() {
        let nibble = ((v >> (i * 4)) & 0xf) as usize;
        let b = [HEX[nibble]];
        uart_write(&b);
    }
}

fn uart_write(bytes: &[u8]) {
    crate::uart::write_bytes(bytes);
}

/// EL1 IRQ (idle WFI wake): ack timer, return.
#[unsafe(naked)]
unsafe extern "C" fn el1_irq_entry() {
    naked_asm!(
        "sub sp, sp, #256",
        "stp x0, x1, [sp, #0]",
        "stp x2, x3, [sp, #16]",
        "stp x4, x5, [sp, #32]",
        "stp x6, x7, [sp, #48]",
        "stp x8, x9, [sp, #64]",
        "stp x10, x11, [sp, #80]",
        "stp x12, x13, [sp, #96]",
        "stp x14, x15, [sp, #112]",
        "stp x16, x17, [sp, #128]",
        "stp x18, x30, [sp, #144]",
        "bl {handler}",
        "ldp x0, x1, [sp, #0]",
        "ldp x2, x3, [sp, #16]",
        "ldp x4, x5, [sp, #32]",
        "ldp x6, x7, [sp, #48]",
        "ldp x8, x9, [sp, #64]",
        "ldp x10, x11, [sp, #80]",
        "ldp x12, x13, [sp, #96]",
        "ldp x14, x15, [sp, #112]",
        "ldp x16, x17, [sp, #128]",
        "ldp x18, x30, [sp, #144]",
        "add sp, sp, #256",
        "eret",
        handler = sym el1_irq_rust,
    )
}

#[no_mangle]
extern "C" fn el1_irq_rust() {
    let irq = gic::ack();
    if irq < 1020 {
        let _ = timer::handle_irq(irq);
        gic::eoi(irq);
    }
}

/// Save EL0 context, handle SVC, then eret or return to scheduler.
#[unsafe(naked)]
unsafe extern "C" fn el0_sync_entry() {
    naked_asm!(
        "sub sp, sp, #288",
        "stp x0, x1, [sp, #0]",
        "stp x2, x3, [sp, #16]",
        "stp x4, x5, [sp, #32]",
        "stp x6, x7, [sp, #48]",
        "stp x8, x9, [sp, #64]",
        "stp x10, x11, [sp, #80]",
        "stp x12, x13, [sp, #96]",
        "stp x14, x15, [sp, #112]",
        "stp x16, x17, [sp, #128]",
        "stp x18, x19, [sp, #144]",
        "stp x20, x21, [sp, #160]",
        "stp x22, x23, [sp, #176]",
        "stp x24, x25, [sp, #192]",
        "stp x26, x27, [sp, #208]",
        "stp x28, x29, [sp, #224]",
        "str x30, [sp, #240]",
        "mrs x0, sp_el0",
        "str x0, [sp, #248]",
        "mrs x0, elr_el1",
        "str x0, [sp, #256]",
        "mrs x0, spsr_el1",
        "str x0, [sp, #264]",
        "mov x0, sp",
        "bl {handler}",
        // x0 = 0 resume, else yield/exit/preempt → shared bridge
        "cbz x0, 1f",
        "b {return_kernel}",
        "1:",
        "ldr x1, [sp, #264]",
        "msr spsr_el1, x1",
        "ldr x1, [sp, #256]",
        "msr elr_el1, x1",
        "ldr x1, [sp, #248]",
        "msr sp_el0, x1",
        "ldp x0, x1, [sp, #0]",
        "ldp x2, x3, [sp, #16]",
        "ldp x4, x5, [sp, #32]",
        "ldp x6, x7, [sp, #48]",
        "ldp x8, x9, [sp, #64]",
        "ldp x10, x11, [sp, #80]",
        "ldp x12, x13, [sp, #96]",
        "ldp x14, x15, [sp, #112]",
        "ldp x16, x17, [sp, #128]",
        "ldp x18, x19, [sp, #144]",
        "ldp x20, x21, [sp, #160]",
        "ldp x22, x23, [sp, #176]",
        "ldp x24, x25, [sp, #192]",
        "ldp x26, x27, [sp, #208]",
        "ldp x28, x29, [sp, #224]",
        "ldr x30, [sp, #240]",
        "add sp, sp, #288",
        "eret",
        handler = sym el0_sync_rust,
        return_kernel = sym crate::process::return_to_kernel,
    )
}

/// EL0 IRQ — same save layout and return bridge as SVC (SCRUM-21).
#[unsafe(naked)]
unsafe extern "C" fn el0_irq_entry() {
    naked_asm!(
        "sub sp, sp, #288",
        "stp x0, x1, [sp, #0]",
        "stp x2, x3, [sp, #16]",
        "stp x4, x5, [sp, #32]",
        "stp x6, x7, [sp, #48]",
        "stp x8, x9, [sp, #64]",
        "stp x10, x11, [sp, #80]",
        "stp x12, x13, [sp, #96]",
        "stp x14, x15, [sp, #112]",
        "stp x16, x17, [sp, #128]",
        "stp x18, x19, [sp, #144]",
        "stp x20, x21, [sp, #160]",
        "stp x22, x23, [sp, #176]",
        "stp x24, x25, [sp, #192]",
        "stp x26, x27, [sp, #208]",
        "stp x28, x29, [sp, #224]",
        "str x30, [sp, #240]",
        "mrs x0, sp_el0",
        "str x0, [sp, #248]",
        "mrs x0, elr_el1",
        "str x0, [sp, #256]",
        "mrs x0, spsr_el1",
        "str x0, [sp, #264]",
        "mov x0, sp",
        "bl {handler}",
        "cbz x0, 1f",
        "b {return_kernel}",
        "1:",
        "ldr x1, [sp, #264]",
        "msr spsr_el1, x1",
        "ldr x1, [sp, #256]",
        "msr elr_el1, x1",
        "ldr x1, [sp, #248]",
        "msr sp_el0, x1",
        "ldp x0, x1, [sp, #0]",
        "ldp x2, x3, [sp, #16]",
        "ldp x4, x5, [sp, #32]",
        "ldp x6, x7, [sp, #48]",
        "ldp x8, x9, [sp, #64]",
        "ldp x10, x11, [sp, #80]",
        "ldp x12, x13, [sp, #96]",
        "ldp x14, x15, [sp, #112]",
        "ldp x16, x17, [sp, #128]",
        "ldp x18, x19, [sp, #144]",
        "ldp x20, x21, [sp, #160]",
        "ldp x22, x23, [sp, #176]",
        "ldp x24, x25, [sp, #192]",
        "ldp x26, x27, [sp, #208]",
        "ldp x28, x29, [sp, #224]",
        "ldr x30, [sp, #240]",
        "add sp, sp, #288",
        "eret",
        handler = sym el0_irq_rust,
        return_kernel = sym crate::process::return_to_kernel,
    )
}

fn load_frame(sp: *mut u64) -> TrapFrame {
    let mut tf = TrapFrame::zero();
    unsafe {
        for i in 0..31 {
            tf.x[i] = *sp.add(i);
        }
        tf.sp_el0 = *sp.add(31);
        tf.elr_el1 = *sp.add(32);
        tf.spsr_el1 = *sp.add(33);
    }
    tf
}

fn store_frame_stack(sp: *mut u64, tf: &TrapFrame) {
    unsafe {
        for i in 0..31 {
            *sp.add(i) = tf.x[i];
        }
        *sp.add(31) = tf.sp_el0;
        *sp.add(32) = tf.elr_el1;
        *sp.add(33) = tf.spsr_el1;
    }
}

#[no_mangle]
extern "C" fn el0_sync_rust(sp: *mut u64) -> u64 {
    let mut tf = load_frame(sp);

    let esr: u64;
    unsafe {
        asm!("mrs {0}, esr_el1", out(reg) esr, options(nostack));
    }
    let ec = (esr >> 26) & 0x3f;
    let action = if ec == 0x15 {
        let nr = tf.x[8];
        let ret = syscall::dispatch_trap(nr, &mut tf);
        tf.x[0] = ret as u64;
        match nr {
            syscall::SYS_YIELD => TrapAction::Yield,
            syscall::SYS_EXIT => TrapAction::Exit,
            _ => TrapAction::Resume,
        }
    } else {
        console::println("exception: unexpected EL0 sync");
        console::print("  esr=");
        print_hex(esr);
        console::print(" elr=");
        print_hex(tf.elr_el1);
        let far: u64;
        unsafe {
            asm!("mrs {0}, far_el1", out(reg) far, options(nostack));
        }
        console::print(" far=");
        print_hex(far);
        console::println("");
        TrapAction::Exit
    };

    process::store_frame(&tf);
    store_frame_stack(sp, &tf);
    action.as_code()
}

#[no_mangle]
extern "C" fn el0_irq_rust(sp: *mut u64) -> u64 {
    let tf = load_frame(sp);
    let irq = gic::ack();
    let mut action = TrapAction::Resume;
    if irq < 1020 {
        if timer::handle_irq(irq) {
            action = TrapAction::Preempt;
        }
        gic::eoi(irq);
    }
    process::store_frame(&tf);
    store_frame_stack(sp, &tf);
    action.as_code()
}

pub fn init() {
    let vbar = exception_vectors as *const () as usize as u64;
    unsafe {
        asm!("msr vbar_el1, {0}", in(reg) vbar, options(nostack));
        asm!("isb", options(nostack));
    }
    console::println("exception: VBAR_EL1 installed (SVC+IRQ)");
}
