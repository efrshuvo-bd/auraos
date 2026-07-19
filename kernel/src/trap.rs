//! Trap frame shared by exception entry and process entry.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub x: [u64; 31], // x0-x30
    pub sp_el0: u64,
    pub elr_el1: u64,
    pub spsr_el1: u64,
}

impl TrapFrame {
    pub const fn zero() -> Self {
        Self {
            x: [0; 31],
            sp_el0: 0,
            elr_el1: 0,
            spsr_el1: 0,
        }
    }
}

/// Result of handling a trap/IRQ from EL0.
///
/// Action codes returned to the asm bridge (`return_to_kernel`):
/// 0=Resume, 1=Yield, 2=Exit, 3=Preempt — all non-zero share one return path.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TrapAction {
    Resume,
    Yield,
    Exit,
    Preempt,
}

impl TrapAction {
    pub fn as_code(self) -> u64 {
        match self {
            TrapAction::Resume => 0,
            TrapAction::Yield => 1,
            TrapAction::Exit => 2,
            TrapAction::Preempt => 3,
        }
    }
}
