//! EL0 process create / enter / leave.

use crate::console;
use crate::elf;
use crate::frame::{self, PAGE_SIZE};
use crate::trap::{TrapAction, TrapFrame};
use crate::vm;
use core::arch::{asm, naked_asm};
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

const MAX_PROCS: usize = 8;
const USER_STACK_TOP: usize = 0x0000_0000_0080_0000;
const USER_STACK_PAGES: usize = 4;
const BRIDGE_STACK_SIZE: usize = 64 * 1024;

#[derive(Clone, Copy)]
pub struct Process {
    pub name: &'static str,
    pub ttbr0: usize,
    pub frame: TrapFrame,
    pub alive: bool,
}

static mut PROCS: [Process; MAX_PROCS] = [Process {
    name: "",
    ttbr0: 0,
    frame: TrapFrame::zero(),
    alive: false,
}; MAX_PROCS];

static PROC_COUNT: AtomicUsize = AtomicUsize::new(0);
static CURRENT: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_ACTION: AtomicU64 = AtomicU64::new(0);

#[repr(C, align(16))]
struct BridgeStack([u8; BRIDGE_STACK_SIZE]);
#[used]
static mut BRIDGE_STACK: BridgeStack = BridgeStack([0; BRIDGE_STACK_SIZE]);

pub fn init() {
    PROC_COUNT.store(0, Ordering::SeqCst);
    CURRENT.store(usize::MAX, Ordering::SeqCst);
}

pub fn spawn(name: &'static str, image: &[u8]) -> bool {
    let idx = PROC_COUNT.load(Ordering::Relaxed);
    if idx >= MAX_PROCS {
        return false;
    }
    let ttbr0 = match vm::create_address_space() {
        Some(t) => t,
        None => {
            console::println("process: ttbr0 alloc failed");
            return false;
        }
    };
    let loaded = match elf::load(ttbr0, image) {
        Some(l) => l,
        None => {
            console::println("process: elf load failed");
            return false;
        }
    };
    if !map_user_stack(ttbr0) {
        console::println("process: stack map failed");
        return false;
    }

    let mut frame = TrapFrame::zero();
    frame.elr_el1 = loaded.entry as u64;
    frame.sp_el0 = USER_STACK_TOP as u64;
    frame.spsr_el1 = 0x3c0;

    unsafe {
        PROCS[idx] = Process {
            name,
            ttbr0,
            frame,
            alive: true,
        };
    }
    PROC_COUNT.fetch_add(1, Ordering::SeqCst);
    true
}

fn map_user_stack(ttbr0: usize) -> bool {
    let mut top = USER_STACK_TOP;
    for _ in 0..USER_STACK_PAGES {
        top -= PAGE_SIZE;
        let phys = match frame::alloc_frame() {
            Some(p) => p,
            None => return false,
        };
        if !vm::map_user_page(ttbr0, top, phys, vm::UserMap::Data) {
            return false;
        }
    }
    true
}

pub fn count() -> usize {
    PROC_COUNT.load(Ordering::Relaxed)
}

pub fn name_at(idx: usize) -> &'static str {
    unsafe { PROCS[idx].name }
}

pub fn is_alive(idx: usize) -> bool {
    unsafe { PROCS[idx].alive }
}

pub fn store_frame(tf: &TrapFrame) {
    let idx = CURRENT.load(Ordering::Relaxed);
    if idx < MAX_PROCS {
        unsafe {
            PROCS[idx].frame = *tf;
        }
    }
}

/// Run process until yield or exit. Uses a bridge stack for the trap return path.
pub fn run(idx: usize) -> TrapAction {
    CURRENT.store(idx, Ordering::SeqCst);
    let (ttbr0, frame_ptr) = unsafe {
        let p = &raw mut PROCS[idx];
        ((*p).ttbr0, &raw const (*p).frame as *const TrapFrame)
    };
    vm::switch_ttbr0(ttbr0);
    unsafe {
        enter_user_asm(frame_ptr);
    }
    // Unreachable — trap path jumps to bridge_from_el0.
    TrapAction::Exit
}

/// Called from exception return path with x0 = action code (1=yield, 2=exit).
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn return_to_kernel() {
    naked_asm!(
        // Switch to dedicated bridge stack, then finish in Rust.
        "adrp x1, {STACK}",
        "add x1, x1, :lo12:{STACK}",
        "add x1, x1, #{SIZE}",
        "mov sp, x1",
        "b {finish}",
        STACK = sym BRIDGE_STACK,
        SIZE = const BRIDGE_STACK_SIZE,
        finish = sym bridge_from_el0,
    )
}

#[no_mangle]
extern "C" fn bridge_from_el0(action: u64) -> ! {
    LAST_ACTION.store(action, Ordering::SeqCst);
    let idx = CURRENT.load(Ordering::Relaxed);
    vm::switch_ttbr0(vm::kernel_ttbr0());
    if action == 2 && idx < MAX_PROCS {
        unsafe {
            PROCS[idx].alive = false;
        }
    }
    CURRENT.store(usize::MAX, Ordering::SeqCst);
    // Re-enter the scheduler on the bridge stack.
    crate::sched::run()
}

#[unsafe(naked)]
unsafe extern "C" fn enter_user_asm(_frame: *const TrapFrame) -> ! {
    naked_asm!(
        "mov x21, x0",
        "ldr x9, [x21, #248]",
        "msr sp_el0, x9",
        "ldr x9, [x21, #256]",
        "msr elr_el1, x9",
        "ldr x9, [x21, #264]",
        "msr spsr_el1, x9",
        "ldp x0, x1, [x21, #0]",
        "ldp x2, x3, [x21, #16]",
        "ldp x4, x5, [x21, #32]",
        "ldp x6, x7, [x21, #48]",
        "ldp x8, x9, [x21, #64]",
        "ldp x10, x11, [x21, #80]",
        "ldp x12, x13, [x21, #96]",
        "ldp x14, x15, [x21, #112]",
        "ldp x16, x17, [x21, #128]",
        "ldp x18, x19, [x21, #144]",
        "ldr x20, [x21, #160]",
        "ldp x22, x23, [x21, #176]",
        "ldp x24, x25, [x21, #192]",
        "ldp x26, x27, [x21, #208]",
        "ldp x28, x29, [x21, #224]",
        "ldr x30, [x21, #240]",
        "ldr x21, [x21, #168]",
        "eret",
    )
}

/// Idle helper if enter_user somehow returns.
#[allow(dead_code)]
fn hang() -> ! {
    loop {
        unsafe {
            asm!("wfi", options(nomem, nostack));
        }
    }
}
