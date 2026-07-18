//! Minimal identity-map stub for early AuraOS (QEMU virt).
//! Full EL1 page-table install comes in a later bring-up; for v0 we document
//! that the bootloader/QEMU already provides a usable identity mapping.

use crate::console;

pub fn init_identity_map() {
    // Placeholder for TTBR0/TTBR1 programming.
    // On QEMU `-kernel`, firmware often leaves a flat map; we record readiness.
    console::println("vm: identity map assumed (QEMU early boot)");
}

pub fn map_user_page(_virt: usize, _phys: usize) -> bool {
    // Future: allocate page table entries for userspace.
    true
}
