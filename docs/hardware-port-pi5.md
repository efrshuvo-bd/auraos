# Hardware port: Raspberry Pi 5

Phase 5 / Sprint 6 bring-up target for AuraOS (Tier C research board → Tier A cloud agent).

Jira: [SCRUM-30](https://auramislab.atlassian.net/browse/SCRUM-30) under epic [SCRUM-12](https://auramislab.atlassian.net/browse/SCRUM-12).  
In-tree board notes: `kernel/src/board_pi5.rs` (constants + feature flags only — **not** a working Pi 5 driver).  
Linked from the [Development Plan](https://auramislab.atlassian.net/wiki/spaces/AuraOS/pages/295074) and [`docs/hardware.md`](hardware.md).

## Why Pi 5

- Documented boot flow (EEPROM → firmware → kernel)
- aarch64, enough RAM for Agent Core host-style userspace during port
- Community device trees / UART

## Bring-up checklist (bootloader → UART → kernel)

Use this as the living checklist. Items marked **done** are documentation / research only unless noted.

### Bootloader / firmware

| Step | Status | Notes |
|------|--------|-------|
| Identify EEPROM / firmware chain for custom kernel load | Research | Pi 5 boots via RP1 + VideoCore firmware; AuraOS currently expects QEMU Linux raw `-kernel` at `0x40080000` |
| Decide load path (custom `config.txt` / `kernel*.img` vs UEFI) | Open | Prefer documented path with serial early; UEFI optional later |
| Confirm aarch64 EL2/EL1 entry expectations | Open | QEMU virt enters EL1 with FDT in `x0`; Pi firmware may differ |
| Document DT blob delivery to kernel | Open | Need `/memory` + UART + GIC nodes; QEMU uses FDT from `-machine virt` |

### UART (early console)

| Step | Status | Notes |
|------|--------|-------|
| Document debug UART pinout / connector | Partial | See `board_pi5::UART_*` hints — verify against current Pi 5 docs before wiring |
| Map PL011 (or SoC UART) MMIO base from DT | Open | QEMU uses PL011 at `0x0900_0000`; Pi 5 base differs |
| Bring up `uart::init` against board MMIO | Deferred | Do **not** claim QEMU PL011 driver works on Pi 5 |
| Print `AuraOS kernel online` on real hardware | Deferred | Milestone gate |

### Kernel load / memory

| Step | Status | Notes |
|------|--------|-------|
| Parse memory map from DT into `frame::init` | Open | QEMU hardcodes frame pool at `0x4400_0000` |
| Adjust linker / load address for Pi image layout | Open | `boot/` + `kernel` linker script today assume QEMU virt |
| Timer + GIC (or GICv3) bring-up | Open | Sprint 2 path is GICv2 + CNTP on virt; Pi 5 is GICv2/v3 in DT — must re-probe |
| Storage for A/B slots (`ota/slots.json` semantics) | Open | Needs SD/eMMC or VirtIO-blk equivalent — see OTA docs |
| Network for cloud Agent Core + OTA | Later | Not required for first serial milestone |

## Port matrix (QEMU virt vs Pi 5)

Clearer view of what is portable vs board-specific. **Do not** treat any Pi column as “done on silicon.”

| Subsystem | QEMU `virt` today | Pi 5 research | Feature flag (`board_pi5::features`) | Next driver task |
|-----------|-------------------|---------------|--------------------------------------|------------------|
| Early console | PL011 `0x0900_0000` | Debug UART from DT | `UART_EARLY_CONSOLE` | Map DT UART → `uart::init`; keep QEMU path default |
| Kernel entry | Raw `-kernel` @ `0x40080000`, FDT in `x0` | Firmware `kernel*.img` + DTB | — | Package image for Pi boot chain; verify EL |
| Memory | Hardcoded pool `0x4400_0000` | `/memory` in DT | `DT_MEMORY_MAP` | DT walker → `frame::init` |
| Interrupts | GICv2 virt defaults | DT GIC v2/v3 | `GIC_FROM_DT` | Re-probe distributor/CPU/redistributor |
| Timer | CNTP PPI 30 | Architected timer; IRQ routing differs | (with GIC) | Confirm PPI/SPI after GIC |
| Display | ramfb / VirtIO-GPU probe | HDMI / VC4 later | — | Out of first serial milestone |
| Storage / A/B | VirtIO-blk **sector0 read** + AURAAB header on QEMU | SD/eMMC partitions | `STORAGE_AB_SLOTS` | Full slot write + Pi storage |
| OTA apply | Host verify + kernel **slot-switch stub** (no write) | Same metadata | `OTA_ON_DEVICE_APPLY` | On-device verify + inactive-slot write |

Boot status line on QEMU today:  
`board: qemu-virt (pi5 research stubs present; not a hardware driver)`  
(`board_pi5::status_line()` — always QEMU virt until a dedicated Pi image exists.)

## Gaps vs QEMU `virt` (must not paper over)

| Area | QEMU `virt` (today) | Pi 5 (target) |
|------|---------------------|---------------|
| Machine | `-machine virt,gic-version=2` | BCM2712 + RP1, vendor firmware |
| Early console | PL011 `0x0900_0000` | Board UART from DT (different base) |
| Kernel entry | Raw `-kernel` @ `0x40080000`, FDT in `x0` | Firmware-defined; may need `kernel*.img` packaging |
| Interrupts | GICv2 distributor/CPU iface at virt defaults | DT-described GIC; verify version |
| Timer | CNTP PPI 30 via virt GIC | Architected timer still OK in principle; IRQ routing differs |
| Display | `ramfb` / VirtIO-GPU probe (Sprint 5) | HDMI / VC4 path — separate from QEMU smoke UI |
| Storage | initrd cpio only (+ VirtIO-blk probe log) | SD/eMMC + A/B partitions for OTA |
| Guests | `-initrd` cpio newc | Same userspace possible once storage/console exist |

**Honesty rule:** `board_pi5` and this doc describe research + compile-time identity. They do **not** enable a fake “runs on Pi 5” code path. Default build remains QEMU virt. No claim that Pi UART/GIC works on real silicon.

## Minimum for “AuraOS on Pi 5” milestone

- Boots to serial: `AuraOS kernel online`
- Scheduler runs `init` / `agent.core` / `shell` tasks (or host-nfs userspace)
- OTA metadata can express A/B slot state (`ota/slots.json`); on-device apply still later

## Related docs

- Tier floors: [hardware.md](hardware.md)
- OTA / 4-year updates: [updates-4y.md](updates-4y.md), [`ota/`](../ota/)
- Architecture Phase 5 notes: [architecture.md](architecture.md)

## Not in v0

- Full GPU compositor, camera, or vendor NPU acceleration (Tier B needs a phone SoC class device)
- Claiming production verified boot on Pi without HSM-backed keys
- Fake hardware drivers that pretend UART/GIC/storage work on Pi 5
