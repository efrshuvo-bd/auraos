# AuraOS architecture

AuraOS is a research **agentic mobile OS**: boot → kernel → `init` → **Agent Core** (required) → shell/apps.

Tracks `devel` through Sprint 6 (board bring-up research + OTA A/B skeleton). Sprint 1 = VirtIO console; Sprint 2 = GICv2 + CNTP preempt; Sprint 3 = initrd guests; Sprint 4 = EL0 Agent Core; Sprint 5 = ramfb / VirtIO-GPU probe + agent UI surface; Sprint 6 = Pi 5 checklist + OTA verify stub ([SCRUM-12](https://auramislab.atlassian.net/browse/SCRUM-12)).

## Layers

1. **Boot** — QEMU `-kernel build/aura-kernel.bin` (raw; Linux boot path) at `0x40080000` + `-initrd` (`-machine virt,gic-version=2`); UEFI later. ELF `-kernel` skips initrd/FDT.
2. **Kernel (`aura-kernel`)** — UART early console, heap, frame allocator, **EL1 identity MMU**, VBAR with **SVC + IRQ**, **GICv2** + **CNTP** (100 Hz) preemption, process table (PID + Runnable/Running/Blocked/Exited), syscalls (`write`/`read`/`yield`/`exit`/`ipc_*`), in-kernel IPC, **ELF64 loader**, FDT `/chosen` initrd discovery, **cpio newc** guest lookup, VirtIO-MMIO console **TX + polled RX**, **display** (VirtIO-GPU probe + QEMU ramfb smoke fill).
3. **Userspace**
   - **Guest EL0** (`userspace/guest`) — `guest-init` / `guest-agent` / `guest-shell` packed into `build/initrd.cpio` by `scripts/pack-initrd.ps1` (not embedded in the kernel image). Shell presents a serial Home + Agent overlay and triggers tools over IPC.
   - **Host demos** — `aura-init` / `aura-agent` / `aura-shell` (Tokio + TCP); PPM framebuffer sketch in `userspace/shell` is the visual contract for Phase 4.
4. **Shared** — length-prefixed JSON IPC + tool schemas + OTA channel/manifest types (host path).

## Bring-up sequence

1. UART → `AuraOS kernel online` (`_start` saves QEMU FDT `x0`)
2. Parse FDT `/chosen` → `linux,initrd-start` / `linux,initrd-end`
3. Heap + frame pool (`0x4400_0000`, 64 MiB)
4. Identity MMU
5. VBAR (SVC + IRQ)
6. VirtIO console probe (TX/RX queues)
7. Display probe (VirtIO-GPU id 16 and/or ramfb via fw_cfg) + optional FB smoke draw
8. GICv2 + CNTP arm
9. Load guests from initrd cpio → `sched::run`

Acceptance: QEMU serial reaches `sched: idle` (see `docs/expected-qemu-serial.txt`). GUI path: `scripts/run-qemu-gui.ps1`.

## I/O paths

| Path | Device | Used by |
|------|--------|---------|
| Early boot / panic / kernel `console` | PL011 UART `0x0900_0000` | EL1 only |
| Guest ELF delivery | QEMU `-initrd` (cpio newc) | Boot only |
| Guest `SYS_WRITE` | VirtIO console TX (MMIO `0x0a00_0000`+) | EL0 via syscall; UART fallback |
| Guest `SYS_READ` | VirtIO console RX (**polled**) | EL0; IRQ→GIC deferred |
| Display (Sprint 5) | QEMU `ramfb` via fw_cfg `etc/ramfb` + VirtIO-GPU MMIO probe | EL1 smoke fill; full GPU queues deferred |
| Timer preempt | CNTP → GICv2 PPI 30 | EL0 mid-run → `TrapAction::Preempt` |

## Trap / preempt return path

EL0 SVC and EL0 IRQ share one save layout and the same bridge:

`exception entry → store TrapFrame → action code → return_to_kernel → bridge_from_el0 → sched::run`

| Action | Code | Process state |
|--------|------|----------------|
| Resume | 0 | stay Running (`eret`) |
| Yield | 1 | → Runnable |
| Exit | 2 | → Exited (slot reusable by later `spawn`) |
| Preempt (CNTP) | 3 | → Runnable |

## Syscall ABI

AAPCS64: **x8 = number**, args in **x0…**, return in **x0**, `svc #0`.

| # | Name | Args |
|---|------|------|
| 1 | `SYS_WRITE` | ptr, len |
| 2 | `SYS_YIELD` | — |
| 3 | `SYS_EXIT` | — |
| 4 | `SYS_IPC_SEND` | channel, payload |
| 5 | `SYS_IPC_RECV` | channel |
| 6 | `SYS_READ` | ptr, len (non-blocking VirtIO RX) |

## Agent as OS primitive

- Started immediately after IPC is ready; `init` fails closed if Agent Core cannot start.
- User-facing actions prefer **tool mediation** (`help`, `system_status`, `list_services`, `echo`).
- **Guest EL0 (Sprint 4):** `guest-agent` runs a resident tool loop over mailbox IPC; shell requests at least `help` + `system_status`. See `docs/agent-core.md`.
- Cloud LLM optional (`AURA_LLM_*`) on the **host** path; built-in tools work offline on guest.
- Kernel stays small; policy + intelligence live in Agent Core.

## Host vs QEMU

| Path | What runs |
|------|-----------|
| `cargo run -p aura-shell` | Full agentic demo on host (auto-starts agent) |
| `cargo run -p aura-init` | init → agent → shell |
| `scripts/build-kernel.ps1` then `scripts/run-qemu.ps1` | Headless kernel + initrd (`-nographic`, virtconsole mux) |
| `scripts/run-qemu-gui.ps1` | Same + `-device ramfb` + host display (GTK preferred on Windows; `-DisplayBackend`); `-VirtioGpu` adds probe-only `virtio-gpu-device` |

## Phase 4 notes (Sprint 5)

- **Always-on agent UI:** guest shell prints a Home + Agent status/prompt panel on serial and invokes `help` / `system_status` from that UI path; host `aura-shell` keeps the richer REPL + 480×800 PPM sketch (`framebuffer.rs`).
- **Display:** kernel `display::init` probes VirtIO-GPU (device id 16) when present and, when `etc/ramfb` exists, DMA-writes a 28-byte big-endian `RAMFBCfg` (addr/fourcc/flags/w/h/stride) via fw_cfg, then paints a 480×800 XRGB8888 smoke surface (Home/Agent labels use an 8×8 bitmap font at 2× scale). DATA-register fw_cfg stores do not activate ramfb. VirtIO-GPU queues / scanout are deferred. GUI default is ramfb-only; `-VirtioGpu` may show QEMU's uninitialized-GPU placeholder until scanout exists.
- **Host display backend:** on Windows, prefer QEMU `-display gtk` (Scoop SDL often hangs at host bring-up with no guest serial). GTK pixbuf/Adwaita warnings are cosmetic. Success = visible smoke paint + serial `ramfb smoke ok`. Override with `-DisplayBackend gtk|sdl|default`.
- **QEMU flags:** documented in `scripts/run-qemu-gui.ps1` (gui) and `scripts/run-qemu.ps1` (headless).

## Phase 5 notes (Sprint 6)

Epic [SCRUM-12](https://auramislab.atlassian.net/browse/SCRUM-12) — Device bring-up & OTA skeleton.

- **Board bring-up (SCRUM-30):** Tier C research target is Raspberry Pi 5. Checklist, port matrix, QEMU gaps, and next driver tasks live in [`docs/hardware-port-pi5.md`](hardware-port-pi5.md). Kernel `board_pi5` holds research constants, compile-time feature flags (all off), and prints an honest status line; it is **not** a working Pi 5 driver. Default runtime remains QEMU `virt`.
- **OTA A/B (SCRUM-31):** Channels `os` / `agent` / `models` in `ota/channels.json` and `shared::ota`; A/B state in `ota/slots.json`; rollback in `ota/apply_update.md`. Host `aura-ota-verify` uses shared `verify_manifest` and rejects unsigned manifests (dev token `dev-signed` only). Kernel logs `ota: A/B not applied`. VirtIO-blk probe stub only. Aligns with [`docs/updates-4y.md`](updates-4y.md).
- **Docs sync (SCRUM-32):** Development Plan + Confluence Architecture link epic SCRUM-12 and these Phase 5 notes.

Deferred: real Pi UART/GIC drivers, on-device OTA apply, production signatures / verified boot, VirtIO-blk backed A/B slots.

## Next kernel milestones

- VirtIO console IRQ → GIC (RX still polled)
- VirtIO-blk driver for mutable/persistent A/B storage (probe stub landed; initrd remains boot path)
- ~~Real EL0 port of Agent Core tool loop~~ — Sprint 4 (mailbox opcodes; richer framing later)
- ~~VirtIO-GPU / framebuffer foundations~~ — Sprint 5 (probe + ramfb smoke; full GPU later)
- ~~Pi 5 bring-up checklist + honest stubs~~ — Sprint 6 research (UART/DT still open)
- ~~OTA channels + host reject-unsigned~~ — Sprint 6 skeleton (on-device apply + real crypto deferred)
- Guest process wait / init-owned spawn for stronger fail-closed
- Richer guest shell input loop (typed prompts on VirtIO console RX)
