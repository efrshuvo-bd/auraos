//! Minimal GICv2 for QEMU virt (`-machine virt,gic-version=2`).

const GICD_BASE: usize = 0x0800_0000;
const GICC_BASE: usize = 0x0801_0000;

const GICD_CTLR: usize = 0x000;
const GICD_ISENABLER: usize = 0x100;
const GICD_ITARGETSR: usize = 0x800;
const GICD_IPRIORITYR: usize = 0x400;

const GICC_CTLR: usize = 0x000;
const GICC_PMR: usize = 0x004;
const GICC_IAR: usize = 0x00c;
const GICC_EOIR: usize = 0x010;

/// Non-secure physical timer PPI on QEMU virt.
pub const IRQ_CNTP: u32 = 30;

#[inline]
unsafe fn r32(base: usize, off: usize) -> u32 {
    core::ptr::read_volatile((base + off) as *const u32)
}

#[inline]
unsafe fn w32(base: usize, off: usize, val: u32) {
    core::ptr::write_volatile((base + off) as *mut u32, val);
}

pub fn init() {
    unsafe {
        // Distributor + CPU interface on.
        w32(GICD_BASE, GICD_CTLR, 1);
        w32(GICC_BASE, GICC_CTLR, 1);
        w32(GICC_BASE, GICC_PMR, 0xff);

        enable_irq_inner(IRQ_CNTP);
    }
    crate::console::println("gic: v2 ready (CNTP PPI 30)");
}

/// Enable an SPI/PPI in the distributor (CPU0 target for SPIs).
pub fn enable_irq(irq: u32) {
    unsafe {
        enable_irq_inner(irq);
    }
}

unsafe fn enable_irq_inner(irq: u32) {
    let reg = (irq / 32) as usize;
    let bit = irq % 32;
    let isen = GICD_ISENABLER + reg * 4;
    let cur = r32(GICD_BASE, isen);
    w32(GICD_BASE, isen, cur | (1 << bit));

    // Priority (lower value = higher priority).
    let pri_off = GICD_IPRIORITYR + irq as usize;
    core::ptr::write_volatile((GICD_BASE + pri_off) as *mut u8, 0xa0);

    // PPIs/SGIs: targets are read-only / banked per-CPU; SPIs need ITARGETSR.
    if irq >= 32 {
        let t_off = GICD_ITARGETSR + irq as usize;
        core::ptr::write_volatile((GICD_BASE + t_off) as *mut u8, 0x01);
    }
}

/// Acknowledge IRQ; returns interrupt ID (1023 = spurious).
pub fn ack() -> u32 {
    unsafe { r32(GICC_BASE, GICC_IAR) }
}

pub fn eoi(irq: u32) {
    unsafe {
        w32(GICC_BASE, GICC_EOIR, irq);
    }
}
