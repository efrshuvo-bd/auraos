//! EL1 identity map + per-process TTBR0 helpers (4K granule).

use crate::console;
use crate::frame::{self, PAGE_SIZE};
use core::arch::asm;

const ENTRIES: usize = 512;

/// Normal memory, Inner Shareable, AF set, EL1-only RW, UXN.
const ATTR_KERNEL_RW: u64 = (0 << 2) | (3 << 8) | (1 << 10) | (0 << 6) | (1u64 << 54);
/// Device nGnRE, Outer Shareable, AF, EL1-only RW, UXN|PXN.
const ATTR_DEVICE: u64 = (1 << 2) | (2 << 8) | (1 << 10) | (0 << 6) | (1u64 << 54) | (1u64 << 53);
/// Normal memory for EL0+EL1 RW user data (UXN).
const ATTR_USER_RW: u64 = (0 << 2) | (3 << 8) | (1 << 10) | (1 << 6) | (1u64 << 54);
/// Normal memory for EL0+EL1 RO+X user text.
const ATTR_USER_RX: u64 = (0 << 2) | (3 << 8) | (1 << 10) | (3 << 6);

static mut KERNEL_L0: usize = 0;

#[repr(C, align(4096))]
struct Table([u64; ENTRIES]);

fn alloc_table() -> Option<&'static mut Table> {
    let phys = frame::alloc_frame()?;
    Some(unsafe { &mut *(phys as *mut Table) })
}

fn pte_table(next_phys: usize) -> u64 {
    (next_phys as u64) | 0b11
}

fn pte_block(phys: usize, attrs: u64) -> u64 {
    (phys as u64) | 0b01 | attrs
}

fn pte_page(phys: usize, attrs: u64) -> u64 {
    (phys as u64) | 0b11 | attrs
}

fn map_block_2m(l1: &mut Table, va: usize, pa: usize, attrs: u64) {
    let i1 = (va >> 30) & 0x1ff;
    let i2 = (va >> 21) & 0x1ff;
    if l1.0[i1] & 1 == 0 {
        let l2 = alloc_table().expect("l2 alloc");
        l1.0[i1] = pte_table(l2 as *mut Table as usize);
    }
    let l2_phys = (l1.0[i1] & !0xfff) as usize;
    let l2 = unsafe { &mut *(l2_phys as *mut Table) };
    l2.0[i2] = pte_block(pa, attrs);
}

/// Build a fresh address space with kernel identity + device MMIO.
pub fn create_address_space() -> Option<usize> {
    let l0 = alloc_table()?;
    let l1 = alloc_table()?;
    l0.0[0] = pte_table(l1 as *mut Table as usize);

    let mut va = 0x0800_0000usize;
    while va < 0x1000_0000 {
        map_block_2m(l1, va, va, ATTR_DEVICE);
        va += 2 * 1024 * 1024;
    }

    va = 0x4000_0000;
    while va < 0x6000_0000 {
        map_block_2m(l1, va, va, ATTR_KERNEL_RW);
        va += 2 * 1024 * 1024;
    }

    Some(l0 as *mut Table as usize)
}

pub fn kernel_ttbr0() -> usize {
    unsafe { KERNEL_L0 }
}

pub fn init_identity_map() {
    let l0 = create_address_space().expect("kernel page tables");
    unsafe { KERNEL_L0 = l0 };

    unsafe {
        asm!("msr mair_el1, {0}", in(reg) 0x00_00_00_00_00_00_04FFu64, options(nostack));
        let tcr: u64 = 16 | (0b01 << 8) | (0b01 << 10) | (0b11 << 12) | (0b010 << 32);
        asm!("msr tcr_el1, {0}", in(reg) tcr, options(nostack));
        asm!("msr ttbr0_el1, {0}", in(reg) l0 as u64, options(nostack));
        asm!("isb", options(nostack));
        asm!("tlbi vmalle1", options(nostack));
        asm!("dsb sy", options(nostack));
        asm!("isb", options(nostack));

        let mut sctlr: u64;
        asm!("mrs {0}, sctlr_el1", out(reg) sctlr, options(nostack));
        sctlr |= (1 << 0) | (1 << 2) | (1 << 12);
        asm!("msr sctlr_el1, {0}", in(reg) sctlr, options(nostack));
        asm!("isb", options(nostack));
    }

    console::println("vm: EL1 identity map installed (MMU on)");
}

pub fn switch_ttbr0(ttbr0: usize) {
    unsafe {
        asm!("msr ttbr0_el1, {0}", in(reg) ttbr0 as u64, options(nostack));
        asm!("isb", options(nostack));
        asm!("tlbi vmalle1", options(nostack));
        asm!("dsb sy", options(nostack));
        asm!("isb", options(nostack));
    }
}

#[derive(Clone, Copy)]
pub enum UserMap {
    Text,
    Data,
}

pub fn map_user_page(ttbr0: usize, virt: usize, phys: usize, kind: UserMap) -> bool {
    let attrs = match kind {
        UserMap::Text => ATTR_USER_RX,
        UserMap::Data => ATTR_USER_RW,
    };

    let l0 = unsafe { &mut *(ttbr0 as *mut Table) };
    let i0 = (virt >> 39) & 0x1ff;
    if l0.0[i0] & 1 == 0 {
        let l1 = match alloc_table() {
            Some(t) => t,
            None => return false,
        };
        l0.0[i0] = pte_table(l1 as *mut Table as usize);
    }
    let l1 = unsafe { &mut *((l0.0[i0] & !0xfff) as usize as *mut Table) };

    let i1 = (virt >> 30) & 0x1ff;
    if l1.0[i1] & 1 == 0 {
        let l2 = match alloc_table() {
            Some(t) => t,
            None => return false,
        };
        l1.0[i1] = pte_table(l2 as *mut Table as usize);
    } else if l1.0[i1] & 0b10 == 0 {
        return false; // unexpected block at L1
    }
    let l2 = unsafe { &mut *((l1.0[i1] & !0xfff) as usize as *mut Table) };

    let i2 = (virt >> 21) & 0x1ff;
    if l2.0[i2] & 1 == 0 {
        let l3 = match alloc_table() {
            Some(t) => t,
            None => return false,
        };
        l2.0[i2] = pte_table(l3 as *mut Table as usize);
    } else if l2.0[i2] & 0b10 == 0 {
        // Split an existing 2MB block into an L3 table so we can remap one page.
        if !split_block_to_pages(l2, i2) {
            return false;
        }
    }
    let l3 = unsafe { &mut *((l2.0[i2] & !0xfff) as usize as *mut Table) };
    let i3 = (virt >> 12) & 0x1ff;
    l3.0[i3] = pte_page(phys & !(PAGE_SIZE - 1), attrs);
    true
}

fn split_block_to_pages(l2: &mut Table, i2: usize) -> bool {
    let block = l2.0[i2];
    let block_pa = (block & 0x0000_FFFF_FFFF_F000) as usize;
    let l3 = match alloc_table() {
        Some(t) => t,
        None => return false,
    };
    for i in 0..ENTRIES {
        let pa = block_pa + i * PAGE_SIZE;
        l3.0[i] = pte_page(pa, ATTR_KERNEL_RW);
    }
    l2.0[i2] = pte_table(l3 as *mut Table as usize);
    true
}

/// Tear down a process address space (SCRUM-47).
///
/// Frees user-mapped pages and all page-table frames belonging to this TTBR0.
/// Does **not** free identity-mapped 2MB physical RAM/MMIO (only the tables).
pub fn destroy_address_space(ttbr0: usize) {
    if ttbr0 == 0 || ttbr0 == unsafe { KERNEL_L0 } {
        return;
    }
    let l0 = unsafe { &mut *(ttbr0 as *mut Table) };
    for i0 in 0..ENTRIES {
        let e0 = l0.0[i0];
        if e0 & 1 == 0 {
            continue;
        }
        if e0 & 0b10 == 0 {
            continue; // unexpected block at L0
        }
        let l1_phys = (e0 & !0xfff) as usize;
        destroy_l1(l1_phys);
        frame::free_frame(l1_phys);
        l0.0[i0] = 0;
    }
    frame::free_frame(ttbr0);
    unsafe {
        asm!("tlbi vmalle1", options(nostack));
        asm!("dsb sy", options(nostack));
        asm!("isb", options(nostack));
    }
}

fn destroy_l1(l1_phys: usize) {
    let l1 = unsafe { &mut *(l1_phys as *mut Table) };
    for i1 in 0..ENTRIES {
        let e1 = l1.0[i1];
        if e1 & 1 == 0 {
            continue;
        }
        if e1 & 0b10 == 0 {
            // 1GB block — identity; do not free phys
            l1.0[i1] = 0;
            continue;
        }
        let l2_phys = (e1 & !0xfff) as usize;
        destroy_l2(l2_phys);
        frame::free_frame(l2_phys);
        l1.0[i1] = 0;
    }
}

fn destroy_l2(l2_phys: usize) {
    let l2 = unsafe { &mut *(l2_phys as *mut Table) };
    for i2 in 0..ENTRIES {
        let e2 = l2.0[i2];
        if e2 & 1 == 0 {
            continue;
        }
        if e2 & 0b10 == 0 {
            // 2MB identity block — do not free phys
            l2.0[i2] = 0;
            continue;
        }
        let l3_phys = (e2 & !0xfff) as usize;
        destroy_l3(l3_phys);
        frame::free_frame(l3_phys);
        l2.0[i2] = 0;
    }
}

fn destroy_l3(l3_phys: usize) {
    let l3 = unsafe { &mut *(l3_phys as *mut Table) };
    for i3 in 0..ENTRIES {
        let e3 = l3.0[i3];
        if e3 & 1 == 0 {
            continue;
        }
        // Only free user-mapped pages (AP bits from ATTR_USER_*).
        let ap = (e3 >> 6) & 0b11;
        if ap == 1 || ap == 3 {
            let page_pa = (e3 & 0x0000_FFFF_FFFF_F000) as usize;
            frame::free_frame(page_pa);
        }
        l3.0[i3] = 0;
    }
}
