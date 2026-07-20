//! Board-specific notes for future Raspberry Pi 5 bring-up (Sprint 6–11 /
//! SCRUM-30 / SCRUM-46 / SCRUM-52 / SCRUM-56 / SCRUM-57).
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

/// QEMU virt PL011 base used by `uart.rs` today (default early console).
pub const QEMU_PL011_MMIO_BASE: usize = 0x0900_0000;

/// Research: BCM2712 on-SoC PL011 may exist but is **not** the usual Pi 5 debug
/// console — do not assume QEMU's `0x0900_0000` maps to anything useful on silicon.
pub const BCM2712_PL011_NOTE: &str =
    "BCM2712 PL011 (if present) ≠ QEMU virt PL011 @ 0x0900_0000; prefer RP1 debug UART";

/// Placeholder MMIO base for Pi early console: **unknown until DT bring-up**.
/// Not wired into `uart.rs`. QEMU virt continues to use [`QEMU_PL011_MMIO_BASE`].
pub const UART_MMIO_BASE_UNVERIFIED: Option<usize> = None;

/// Sprint 10–11 research: expected DT path hint for RP1 UART (confirm on silicon DT).
/// Not parsed by `fdt.rs` yet — documentation only.
pub const UART_DT_NODE_HINT: &str = "RP1 serial@… under /rp1 (not QEMU PL011 @ 0x0900_0000)";

/// Compatible string research hint (confirm against Pi 5 DT before coding).
pub const UART_DT_COMPATIBLE_HINT: &str = "arm,pl011 or vendor RP1 UART compatible (confirm on DT)";

/// Target baud for first silicon console experiment (research only).
pub const UART_EARLY_BAUD_HINT: u32 = 115_200;

/// Which early-console map a future Pi image would prefer (selection helper).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EarlyConsoleMap {
    /// Current supported path — `uart.rs` PL011 @ [`QEMU_PL011_MMIO_BASE`].
    QemuPl011,
    /// Future: MMIO from RP1 UART DT node (not implemented).
    Pi5Rp1FromDt,
    /// Explicitly unset — refuse to pretend silicon works.
    Unverified,
}

/// Active early-console map for this binary. Always QEMU PL011 until a Pi image.
pub const ACTIVE_EARLY_CONSOLE: EarlyConsoleMap = EarlyConsoleMap::QemuPl011;

/// Resolve the MMIO base for early console given the research map.
///
/// Returns `Some(QEMU_PL011_MMIO_BASE)` for QEMU; `None` for Pi/unverified until DT.
pub fn early_console_mmio_base(map: EarlyConsoleMap) -> Option<usize> {
    match map {
        EarlyConsoleMap::QemuPl011 => Some(QEMU_PL011_MMIO_BASE),
        EarlyConsoleMap::Pi5Rp1FromDt => UART_MMIO_BASE_UNVERIFIED,
        EarlyConsoleMap::Unverified => None,
    }
}

/// One-line UART research status (honest: not a silicon driver).
pub fn uart_research_status_line() -> &'static str {
    match ACTIVE_EARLY_CONSOLE {
        EarlyConsoleMap::QemuPl011 => {
            "uart: qemu PL011 @ 0x09000000 (pi5 RP1 map research only; UART_EARLY_CONSOLE=false)"
        }
        EarlyConsoleMap::Pi5Rp1FromDt => "uart: pi5 RP1-from-DT profile (incomplete; not wired)",
        EarlyConsoleMap::Unverified => "uart: unverified map (no early console MMIO)",
    }
}

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

/// GICv3 redistributor base from DT — **unset** (Pi may be v3; virt is v2).
pub const GIC_REDIST_BASE_UNVERIFIED: Option<usize> = None;

/// Sprint 10–11 research: next DT walk for GIC (not implemented).
pub const GIC_DT_NODE_HINT: &str =
    "interrupt-controller@… — read reg for distributor + CPU/redistributor; do not hardcode virt";

/// Compatible / cell research hints for a future DT walker (not parsed yet).
pub const GIC_DT_COMPATIBLE_HINT: &str = "arm,gic-400 | arm,gic-v3 (confirm on Pi 5 DT)";
pub const GIC_DT_REG_LAYOUT_HINT: &str =
    "v2: dist + cpu iface; v3: dist + redistributor frame(s) — size from #redistributor-regions";

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

/// True when early console would use unverified Pi MMIO (always false on QEMU build).
pub fn pi5_early_console_ready() -> bool {
    features::UART_EARLY_CONSOLE && early_console_mmio_base(EarlyConsoleMap::Pi5Rp1FromDt).is_some()
}
