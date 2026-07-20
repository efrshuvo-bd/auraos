//! Board-specific notes for future Raspberry Pi 5 bring-up (Sprint 6–10 /
//! SCRUM-30 / SCRUM-46 / SCRUM-52).
//!
//! See `docs/hardware-port-pi5.md`.
//!
//! This module is **research metadata** used by the QEMU virt kernel build.
//! It does **not** implement Pi 5 UART, GIC, or firmware drivers. Constants below
//! are documented targets / hints for the port checklist — do not treat them as
//! verified MMIO maps until a real board bring-up confirms them against DT.

/// Logical board id (Tier C research target). Active runtime remains QEMU virt.
pub const BOARD: &str = "raspberry-pi-5";

/// Human-readable early-console hint for port notes / serial banners.
pub const UART_HINT: &str = "Use Pi 5 debug UART for early console during port";

/// Research label for the debug UART (confirm pinout before soldering).
pub const UART_DEBUG_LABEL: &str = "Pi 5 debug UART (3-pin / dedicated header — verify current docs)";

/// Placeholder MMIO base: **unknown until DT bring-up**. Not wired into `uart.rs`.
/// QEMU virt continues to use PL011 at `0x0900_0000`.
pub const UART_MMIO_BASE_UNVERIFIED: Option<usize> = None;

/// Sprint 10 research: expected DT path hint for RP1 UART (confirm on silicon DT).
/// Not parsed by `fdt.rs` yet — documentation only.
pub const UART_DT_NODE_HINT: &str = "RP1 serial@… under /rp1 (not QEMU PL011 @ 0x0900_0000)";

/// Target CPU architecture for the port (same as QEMU virt).
pub const ARCH: &str = "aarch64";

/// Interrupt controller expectation (must be taken from Pi 5 DT, not virt defaults).
pub const GIC_EXPECTATION: &str = "GICv2 or GICv3 per device tree — re-probe; do not reuse virt MMIO";

/// QEMU virt GICv2 defaults (for contrast only — `gic.rs` hardcodes these today).
pub const QEMU_GICD_BASE: usize = 0x0800_0000;
pub const QEMU_GICC_BASE: usize = 0x0801_0000;

/// Placeholder GIC distributor / CPU interface from DT — **unset** until GIC_FROM_DT.
pub const GIC_DIST_BASE_UNVERIFIED: Option<usize> = None;
pub const GIC_CPU_BASE_UNVERIFIED: Option<usize> = None;

/// Sprint 10 research: next DT walk for GIC (not implemented).
pub const GIC_DT_NODE_HINT: &str =
    "interrupt-controller@… — read reg for distributor + CPU/redistributor; do not hardcode virt";

/// Boot path note for packaging (not implemented).
pub const BOOT_PATH_NOTE: &str =
    "Pi firmware → kernel image + DTB; differs from QEMU raw -kernel @ 0x40080000";

/// Compile-time feature flags for the Pi 5 port.
///
/// All `false` until a dedicated board image wires real drivers. These exist so
/// call sites can gate on named capabilities instead of inventing fake hardware.
pub mod features {
    /// PL011 (or SoC UART) early console on real Pi MMIO.
    pub const UART_EARLY_CONSOLE: bool = false;
    /// Device-tree memory map → frame allocator.
    pub const DT_MEMORY_MAP: bool = false;
    /// GICv2/v3 from Pi DT (not QEMU virt defaults).
    pub const GIC_FROM_DT: bool = false;
    /// SD/eMMC (or equivalent) storage for A/B slots.
    pub const STORAGE_AB_SLOTS: bool = false;
    /// On-device OTA apply path (host verify already exists).
    pub const OTA_ON_DEVICE_APPLY: bool = false;
}

/// Compile-time board profile. Default build is QEMU virt (`BoardProfile::QemuVirt`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardProfile {
    /// Current supported runtime (Sprint 1–5 path).
    QemuVirt,
    /// Future Pi 5 profile — selected only when a real port lands.
    RaspberryPi5Research,
}

/// Active profile for this binary. Always QEMU virt until a dedicated Pi image exists.
pub const ACTIVE_PROFILE: BoardProfile = BoardProfile::QemuVirt;

/// One-line status for console / docs sync (honest about research-only state).
pub fn status_line() -> &'static str {
    match ACTIVE_PROFILE {
        BoardProfile::QemuVirt => {
            "board: qemu-virt (pi5 research stubs present; not a hardware driver)"
        }
        BoardProfile::RaspberryPi5Research => {
            "board: raspberry-pi-5 research profile (incomplete)"
        }
    }
}

/// Whether any real Pi 5 driver feature is enabled (always false today).
pub fn any_pi5_driver_enabled() -> bool {
    features::UART_EARLY_CONSOLE
        || features::DT_MEMORY_MAP
        || features::GIC_FROM_DT
        || features::STORAGE_AB_SLOTS
        || features::OTA_ON_DEVICE_APPLY
}
