//! EL0 process create / enter / leave.
//!
//! Slot policy: `Exited` entries stay in the table until a new `spawn` reuses the
//! first free (`Exited` or never-used) slot. Address spaces of exited processes
//! are not freed yet (bump frame allocator).

use crate::console;
use crate::elf;
use crate::frame::{self, PAGE_SIZE};
use crate::trap::{TrapAction, TrapFrame};
use crate::vm;
use core::arch::{asm, naked_asm};
use core::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};

const MAX_PROCS: usize = 8;
const USER_STACK_TOP: usize = 0x0000_0000_0080_0000;
const USER_STACK_PAGES: usize = 4;
const BRIDGE_STACK_SIZE: usize = 64 * 1024;

/// EL0 SPSR: EL0t, DAIF with IRQ unmasked (I=0) so CNTP can preempt.
const SPSR_EL0_IRQ_ON: u64 = 0x340;

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Blocked reserved for future IPC wait.
pub enum State {
    Runnable,
    Running,
    Blocked,
    Exited,
}

#[derive(Clone, Copy)]
pub struct Process {
    pub pid: u32,
    pub name: &'static str,
    pub state: State,
    pub ttbr0: usize,
    pub frame: TrapFrame,
    /// Exit status from SYS_EXIT (valid when `state == Exited` and pid != 0).
    pub exit_status: i32,
    /// True once waitpid has reaped this exited slot (keeps status until reuse).
    pub reaped: bool,
}

static mut PROCS: [Process; MAX_PROCS] = [Process {
    pid: 0,
    name: "",
    state: State::Exited,
    ttbr0: 0,
    frame: TrapFrame::zero(),
    exit_status: 0,
    reaped: true,
}; MAX_PROCS];

static NEXT_PID: AtomicU32 = AtomicU32::new(1);
static SLOT_COUNT: AtomicUsize = AtomicUsize::new(0);
static CURRENT: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_ACTION: AtomicU64 = AtomicU64::new(0);
static PREEMPT_LOGS: AtomicU32 = AtomicU32::new(0);

/// Last bridge action code (0=none/resume, 1=yield, 2=exit, 3=preempt).
pub fn last_action() -> u64 {
    LAST_ACTION.load(Ordering::Relaxed)
}

#[repr(C, align(16))]
struct BridgeStack([u8; BRIDGE_STACK_SIZE]);
#[used]
static mut BRIDGE_STACK: BridgeStack = BridgeStack([0; BRIDGE_STACK_SIZE]);

pub fn init() {
    SLOT_COUNT.store(0, Ordering::SeqCst);
    CURRENT.store(usize::MAX, Ordering::SeqCst);
    NEXT_PID.store(1, Ordering::SeqCst);
}

pub fn spawn(name: &'static str, image: &[u8]) -> bool {
    spawn_returning_pid(name, image).is_some()
}

/// Spawn a process and return its PID (Sprint 8 / SCRUM-39).
pub fn spawn_returning_pid(name: &'static str, image: &[u8]) -> Option<u32> {
    let idx = match find_slot() {
        Some(i) => i,
        None => {
            console::println("process: table full");
            return None;
        }
    };
    let ttbr0 = match vm::create_address_space() {
        Some(t) => t,
        None => {
            console::println("process: ttbr0 alloc failed");
            return None;
        }
    };
    let loaded = match elf::load(ttbr0, image) {
        Some(l) => l,
        None => {
            console::println("process: elf load failed");
            return None;
        }
    };
    if !map_user_stack(ttbr0) {
        console::println("process: stack map failed");
        return None;
    }

    let pid = NEXT_PID.fetch_add(1, Ordering::SeqCst);
    let mut frame = TrapFrame::zero();
    frame.elr_el1 = loaded.entry as u64;
    frame.sp_el0 = USER_STACK_TOP as u64;
    frame.spsr_el1 = SPSR_EL0_IRQ_ON;

    unsafe {
        PROCS[idx] = Process {
            pid,
            name,
            state: State::Runnable,
            ttbr0,
            frame,
            exit_status: 0,
            reaped: false,
        };
    }
    let used = SLOT_COUNT.load(Ordering::Relaxed);
    if idx >= used {
        SLOT_COUNT.store(idx + 1, Ordering::SeqCst);
    }
    Some(pid)
}

/// Non-blocking wait for an exited process (Sprint 7/8).
///
/// - `pid == 0` or `pid == u32::MAX` (`-1` as u32): first unreaped exited peer
/// - `pid > 0` (and not `MAX`): that pid if exited and unreaped
///
/// Returns `Some((reaped_pid, status))`, or `None` if nothing ready (WNOHANG).
pub fn waitpid_noblock(waiter_pid: u32, pid: u32) -> Option<(u32, i32)> {
    let wait_any = pid == 0 || pid == u32::MAX;
    unsafe {
        for i in 0..SLOT_COUNT.load(Ordering::Relaxed).min(MAX_PROCS) {
            let p = &mut PROCS[i];
            if p.state != State::Exited || p.reaped || p.pid == 0 || p.pid == waiter_pid {
                continue;
            }
            if !wait_any && p.pid != pid {
                continue;
            }
            p.reaped = true;
            return Some((p.pid, p.exit_status));
        }
    }
    None
}

pub fn current_pid() -> u32 {
    let idx = CURRENT.load(Ordering::Relaxed);
    if idx < MAX_PROCS {
        unsafe { PROCS[idx].pid }
    } else {
        0
    }
}

/// Name of the currently running process ("" if none).
pub fn current_name() -> &'static str {
    let idx = CURRENT.load(Ordering::Relaxed);
    if idx < MAX_PROCS {
        unsafe { PROCS[idx].name }
    } else {
        ""
    }
}

/// Record exit status for the current process (called from syscall before bridge).
pub fn set_exit_status(status: i32) {
    let idx = CURRENT.load(Ordering::Relaxed);
    if idx < MAX_PROCS {
        unsafe {
            PROCS[idx].exit_status = status;
        }
    }
}

fn find_slot() -> Option<usize> {
    unsafe {
        for i in 0..MAX_PROCS {
            if PROCS[i].state == State::Exited {
                return Some(i);
            }
        }
    }
    None
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

pub fn slot_count() -> usize {
    SLOT_COUNT.load(Ordering::Relaxed)
}

pub fn name_at(idx: usize) -> &'static str {
    unsafe { PROCS[idx].name }
}

pub fn pid_at(idx: usize) -> u32 {
    unsafe { PROCS[idx].pid }
}

pub fn is_runnable(idx: usize) -> bool {
    unsafe { PROCS[idx].state == State::Runnable }
}

pub fn store_frame(tf: &TrapFrame) {
    let idx = CURRENT.load(Ordering::Relaxed);
    if idx < MAX_PROCS {
        unsafe {
            PROCS[idx].frame = *tf;
        }
    }
}

/// Run process until yield, exit, or preempt. Uses a bridge stack for the return path.
pub fn run(idx: usize) -> TrapAction {
    CURRENT.store(idx, Ordering::SeqCst);
    unsafe {
        PROCS[idx].state = State::Running;
    }
    let (ttbr0, frame_ptr) = unsafe {
        let p = &raw mut PROCS[idx];
        ((*p).ttbr0, &raw const (*p).frame as *const TrapFrame)
    };
    vm::switch_ttbr0(ttbr0);
    unsafe {
        enter_user_asm(frame_ptr);
    }
    TrapAction::Exit
}

/// Called from exception/IRQ return path with x0 = action code (1=yield, 2=exit, 3=preempt).
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn return_to_kernel() {
    naked_asm!(
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

    if idx < MAX_PROCS {
        unsafe {
            match action {
                2 => {
                    PROCS[idx].state = State::Exited;
                }
                1 | 3 => {
                    PROCS[idx].state = State::Runnable;
                    if action == 3 {
                        let n = PREEMPT_LOGS.fetch_add(1, Ordering::Relaxed);
                        if n < 3 {
                            console::print("sched: preempt pid=");
                            print_u32(PROCS[idx].pid);
                            console::println("");
                        }
                    }
                }
                _ => {}
            }
        }
    }
    CURRENT.store(usize::MAX, Ordering::SeqCst);
    crate::sched::run()
}

fn print_u32(mut v: u32) {
    if v == 0 {
        crate::uart::write_bytes(b"0");
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 10;
    while v > 0 {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    crate::uart::write_bytes(&buf[i..]);
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

#[allow(dead_code)]
fn hang() -> ! {
    loop {
        unsafe {
            asm!("wfi", options(nomem, nostack));
        }
    }
}
