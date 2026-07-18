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

/// Result of handling a trap from EL0.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TrapAction {
    Resume,
    Yield,
    Exit,
}
