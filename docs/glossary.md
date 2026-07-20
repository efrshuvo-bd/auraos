# AuraOS Glossary — OS & project vocabulary

Learner-friendly dictionary for terms used heavily in AuraOS docs, code, and sprints.
Plain-language first; **In AuraOS** notes only when the project meaning is narrower or differs from industry use.

Related: [architecture.md](architecture.md) · [agent-core.md](agent-core.md) · [hardware.md](hardware.md) · [updates-4y.md](updates-4y.md) · [hardware-port-pi5.md](hardware-port-pi5.md)

---

## Table of contents

1. [Project framing](#1-project-framing)
2. [Machine & emulation](#2-machine--emulation)
3. [CPU privilege & exceptions](#3-cpu-privilege--exceptions)
4. [Interrupts & timers](#4-interrupts--timers)
5. [Devices & display](#5-devices--display)
6. [Boot & userspace](#6-boot--userspace)
7. [Agent & IPC](#7-agent--ipc)
8. [Updates & product](#8-updates--product)
9. [Process & workflow](#9-process--workflow)

---

## 1. Project framing

**AuraOS** — A from-zero research agentic mobile OS: own boot path, kernel, IPC, and userspace, with Agent Core as a required system service (not an Android fork or launcher overlay).

**From-zero** — Building the OS stack ourselves (boot → kernel → init → agent → shell) instead of forking an existing mobile OS. In AuraOS: the teaching kernel and guest EL0 path on QEMU come first; shipping BSP/drivers come later.

**Research OS / teaching kernel** — A small, readable kernel meant to prove concepts and stay honest about stubs, not a production smartphone image yet. In AuraOS: default runtime is QEMU `virt`; Pi 5 is research stubs only.

**Agentic (agentic OS)** — The AI agent is designed in as a first-class system citizen (started by init, policy-gated tools), not bolted on later as an app. In AuraOS: Agent Core is required; init fails closed if it cannot start.

**Tier A** — Hardware profile for a smooth **cloud-agent** experience (models in the cloud; local Agent Core + shell). See `docs/hardware.md` (e.g. ~6–8 GB RAM floors).

**Tier B** — Hardware profile for a smooth **on-device agent** (local quantized models + tool loops; higher RAM/NPU). In AuraOS: longer-term product target; not required for QEMU bring-up.

**Tier C** — Research / bring-up profile: QEMU `virt` aarch64 today; Raspberry Pi 4/5 as a board research target. In AuraOS: all day-to-day kernel work lands here first.

**Phase / Sprint goal names** — Informal labels for milestone themes (e.g. Phase 4 = mobile shell / display foundations; Phase 5 = board + OTA skeleton). Tracked formally as Jira epics and sprint retros.

---

## 2. Machine & emulation

**QEMU** — Open-source machine emulator/virtualizer. In AuraOS: primary development machine via `scripts/run-qemu.ps1` (headless) and `scripts/run-qemu-gui.ps1` (display).

**virt (QEMU machine)** — QEMU's generic ARM virtual platform (`-machine virt,...`). In AuraOS: `-machine virt,gic-version=2` with raw `-kernel` at `0x40080000` and optional `-initrd`.

**aarch64** — 64-bit Arm architecture (AArch64). In AuraOS: kernel and guest EL0 binaries target aarch64; host demos may also run on the developer PC via Tokio.

**Host** — The real machine (or OS process) running QEMU or cargo demos. In AuraOS: also the Tokio path (`aura-init` / `aura-agent` / `aura-shell`) that exercises Agent Core without the kernel.

**Guest** — Software running *inside* the emulated machine (or EL0 process under the AuraOS kernel). In AuraOS: `guest-init`, `guest-agent`, `guest-shell` packed into `build/initrd.cpio`.

**Emulation vs virtualization** — Emulation recreates hardware in software; virtualization runs a guest with hardware assist. QEMU can do either; AuraOS uses QEMU mainly as a convenient virt board for bring-up.

**PL011 UART** — Common Arm serial UART. In AuraOS: early kernel console at MMIO `0x0900_0000` on QEMU virt (EL1 only); guest I/O prefers VirtIO console.

---

## 3. CPU privilege & exceptions

**Exception level (EL)** — Arm privilege tiers. Higher EL is more privileged. Common ones: EL0 (apps), EL1 (OS kernel), EL2 (hypervisor), EL3 (firmware/secure monitor).

**EL0** — Least-privileged userspace. In AuraOS: guest processes (`guest-init` / `guest-agent` / `guest-shell`) run here and talk to the kernel via syscalls (SVC).

**EL1** — Kernel privilege. In AuraOS: `aura-kernel` runs at EL1 with identity-mapped MMU, exception vectors, drivers, and the scheduler.

**SVC (supervisor call)** — Instruction that traps from EL0 into the kernel to request a service. In AuraOS: `svc #0` with syscall number in `x8` (AAPCS64).

**IRQ** — Interrupt request: asynchronous hardware signal that can interrupt normal execution. In AuraOS: IRQ path shares trap frame layout with SVC; used for timer preempt and (Sprint 7+) VirtIO console RX drain.

**Exception / trap** — CPU diversion to a handler for SVC, IRQ, faults, etc. In AuraOS: entry stores a `TrapFrame`, picks an action (resume / yield / exit / preempt), then returns via a bridge into `sched::run`.

**VBAR (Vector Base Address Register)** — Holds the base of the exception vector table. In AuraOS: VBAR is set so EL0 SVC and IRQ land in kernel handlers.

**MMU (Memory Management Unit)** — Hardware that translates virtual addresses and enforces permissions. In AuraOS: early bring-up uses an **identity MMU** (VA ≈ PA) so devices and RAM stay easy to map.

**TTBR (Translation Table Base Register)** — Points at the page-table root the MMU walks. Mentioned when discussing enabling or switching address spaces; AuraOS's early identity map is the relevant context today.

**Identity map** — Page tables where virtual address equals physical address for a region. In AuraOS: used so UART, VirtIO MMIO, and the frame pool stay reachable without a full userspace VA redesign yet.

**eret** — Arm "exception return" instruction that restores privilege and PC after a trap. In AuraOS: used to resume an EL0 process after SVC/IRQ handling.

**TrapFrame** — Saved CPU registers for a trapped process. In AuraOS: shared layout for EL0 SVC and IRQ return paths.

---

## 4. Interrupts & timers

**GIC (Generic Interrupt Controller)** — Arm's standard interrupt distributor/CPU interface. In AuraOS: **GICv2** on QEMU virt (`gic-version=2`).

**PPI (Private Peripheral Interrupt)** — Per-CPU interrupt ID range on GIC. In AuraOS: CNTP uses **PPI 30** for preemption ticks.

**SPI (Shared Peripheral Interrupt)** — System-wide interrupt IDs (shared across CPUs). In AuraOS: VirtIO-MMIO console IRQ is wired toward GIC (e.g. SPI 48 path in Sprint 7); earlier RX was poll-only.

**CNTP** — Arm generic **physical** timer. In AuraOS: armed at ~100 Hz; IRQ → preempt running EL0 (`TrapAction::Preempt`).

**Preempt / preemption** — Forcibly pausing a running process so another can run (time-slicing). In AuraOS: timer IRQ marks the process Runnable and returns to the scheduler; cooperative `yield` also exists as a syscall.

**Polled I/O** — Driver repeatedly checks device status instead of waiting for an IRQ. In AuraOS: VirtIO console RX started as polled; IRQ drain was added later while poll remains as fallback.

---

## 5. Devices & display

**MMIO (Memory-Mapped I/O)** — Talking to devices by reading/writing special addresses. In AuraOS: VirtIO devices appear on virtio-mmio buses (e.g. console near `0x0a00_0000`).

**VirtIO** — Standard virtual I/O device model for VMs (queues + descriptors). In AuraOS: console, GPU probe, and block (A/B disk) use VirtIO-MMIO on QEMU.

**VirtIO-console** — VirtIO serial console device. In AuraOS: guest `SYS_WRITE` / `SYS_READ` path (TX + RX); UART remains early/panic path.

**VirtIO-blk** — VirtIO block (disk) device. In AuraOS: QEMU attaches `build/ab-slots.img`; kernel can read sector 0 (`AURAAB` header). Full mutable A/B apply write is still a later milestone.

**VirtIO-GPU** — VirtIO graphics device (device id 16). In AuraOS: MMIO probe + control queue + 2D resource create / SET_SCANOUT / flush (Sprint 8); GUI default still prefers ramfb without `-VirtioGpu`.

**ramfb** — QEMU "RAM framebuffer": guest memory painted as the host window. In AuraOS: configured via fw_cfg `etc/ramfb`, 480×800 XRGB8888 smoke fill with Home/Agent glyphs.

**fw_cfg** — QEMU firmware configuration interface (named files the guest can read/write). In AuraOS: ramfb is activated by a **DMA write** of `RAMFBCfg` to `etc/ramfb` (plain DATA-register stores do not activate it).

**DMA (Direct Memory Access)** — Hardware moving data without CPU byte loops. In AuraOS: fw_cfg DMA descriptor path is required to enable ramfb; also a general term for device↔memory transfers.

**Framebuffer** — Memory buffer holding pixels for a display. In AuraOS: kernel smoke surface for ramfb; host `aura-shell` also has a PPM sketch as a visual contract.

**Scanout** — Presenting a framebuffer (or GPU resource) to the actual display pipeline. In AuraOS: full VirtIO-GPU scanout is not done yet; ramfb provides the visible QEMU window for Sprint 5+.

**GTK / SDL** — Host display backends QEMU can use to show a window. In AuraOS: on Windows, prefer GTK (`-display gtk`); Scoop SDL has hung at host bring-up in practice. Override via `-DisplayBackend`.

**Smoke test / smoke fill** — Minimal "does it light up?" check (e.g. solid color + labels). In AuraOS: success string `ramfb smoke ok` on serial plus a visible painted window.

**Probe / stub vs driver** — A **probe** detects a device and logs identity; a **stub** intentionally incomplete path; a **driver** performs real I/O. In AuraOS: honesty rule — Pi 5 and some OTA paths are stubs/research, not shipping drivers.

---

## 6. Boot & userspace

**Boot path** — Steps from power-on / QEMU start until the kernel and first userspace run. In AuraOS: QEMU loads raw kernel + optional initrd; UEFI is future.

**initrd (initial RAM disk)** — Archive passed to the kernel at boot containing early userspace. In AuraOS: `build/initrd.cpio` via QEMU `-initrd`; discovered from FDT `/chosen` (`linux,initrd-start` / `linux,initrd-end`).

**cpio (newc)** — Archive format often used for initrd. In AuraOS: `scripts/pack-initrd.ps1` packs guest ELFs into newc; kernel looks up guests by name.

**FDT / DTB (Flattened Device Tree)** — Binary description of hardware (memory, UART, GIC, …) passed to the kernel. In AuraOS: QEMU provides FDT in `x0` at entry; `/chosen` carries initrd bounds.

**ELF / ELF64** — Executable and Linkable Format. In AuraOS: kernel loads 64-bit guest ELFs from the initrd into process slots.

**init / PID 1** — First userspace process; traditionally responsible for starting the rest of the system. In AuraOS: `guest-init` owns spawn of Agent Core and shell via `SYS_SPAWN` (kernel boots init only).

**PID** — Process identifier. In AuraOS: process table tracks PID plus Runnable / Running / Blocked / Exited.

**Syscall** — Request from userspace into the kernel (via SVC). In AuraOS: `write`, `read`, `yield`, `exit`, `ipc_*`, richer `waitpid` (`SYS_WAITPID`), and init-only `SYS_SPAWN`.

**waitpid** — Wait for a child process to change state (classic Unix). In AuraOS: packed `(pid, status)` return, wait-any (`0` / `-1`), blocking helper; used by init for fail-closed lifecycle.

**spawn** — Create/start a new process. In AuraOS: init loads agent/shell from initrd through `SYS_SPAWN`; kernel no longer hard-starts all three as the sole path.

**Scheduler (`sched`)** — Code that chooses which process runs. In AuraOS: `sched::run` loops until idle; acceptance often includes serial reaching `sched: idle`.

**Userspace** — Code running outside the kernel (EL0 guests or host demos). Contrast with kernel EL1.

---

## 7. Agent & IPC

**Agent Core** — Privileged AuraOS system service (`agent.core`) for session memory, natural-language/tool mediation, policy, and pluggable model backends. See `docs/agent-core.md`.

**Tool / tool loop** — Named actions the agent may run (`help`, `system_status`, `list_services`, `echo`, …). In AuraOS: guest agent runs a resident loop over mailbox IPC; shell expects at least `help` + `system_status`.

**IPC (Inter-Process Communication)** — Message passing between processes. In AuraOS: host uses length-prefixed JSON over TCP `127.0.0.1:7420`; guest EL0 uses in-kernel u64 **mailboxes**.

**Mailbox** — Small fixed channel for posting values between processes. In AuraOS: channels include READY (ch2), request (ch3), response (ch4) with opcodes such as `0xA11E` READY and `0xDEAD` SHUTDOWN.

**Fail-closed** — If a required dependency is missing, refuse a "healthy" session instead of continuing degraded. In AuraOS: init/shell treat missing Agent READY (or failed core tools) as failure (`FAIL CLOSED`).

**Policy gate** — Rules deciding which tools may run for a session. Part of Agent Core's responsibilities on the host path; guest starts with a small built-in tool set.

**Model backend** — Where LLM inference runs (mock, OpenAI-compatible cloud, future on-device). In AuraOS: cloud optional via `AURA_LLM_*` on host; guest built-in tools work offline.

---

## 8. Updates & product

**OTA (Over-The-Air)** — Remote software update delivery. In AuraOS: channels `os` / `agent` / `models`; host `aura-ota-verify` rejects unsigned manifests (`dev-signed` token only today).

**Manifest** — Signed metadata describing an update payload (channel, version, hashes, etc.). In AuraOS: fixtures under `ota/fixtures/`; verify logic in `shared::ota`.

**A/B slots** — Two system partitions/slots; update the inactive one, reboot into it, roll back if boot fails. In AuraOS: metadata in `ota/slots.json`; QEMU disk sector 0 magic `AURAAB`; kernel apply is still a **stub** (logs would-switch / refuses unsigned; no production write yet).

**Rollback** — Revert to the previous good slot after a failed update/boot. Documented in `ota/apply_update.md`; full on-device apply remains incomplete.

**Signed / unsigned** — Cryptographic authenticity of update blobs. In AuraOS: host verify rejects unsigned; production crypto / HSM-backed signatures are deferred.

**Verified boot** — Chain of trust from bootloader → kernel → system that rejects tampered images. In AuraOS: product requirement for shipping devices; **not** implemented as production trust yet.

**4-year support / EOS** — Product commitment: signed OS + Agent Core updates until end-of-support (ship date + 4 years per device generation). See `docs/updates-4y.md`.

**OEM** — Original Equipment Manufacturer (device maker). In AuraOS context: partner/device constraints for BSP life, boot docs, and update windows — not a specific shipping OEM yet.

**Bring-up** — First-time work to get serial, timers, storage, etc. working on a board or machine. In AuraOS: QEMU virt is the daily path; Pi 5 has a research checklist (`docs/hardware-port-pi5.md`).

**BSP (Board Support Package)** — Vendor/board-specific boot and driver glue. Prefer obtainable docs and long life when picking Tier A/B SoCs.

**Channel (`os` / `agent` / `models`)** — Separate OTA streams so Agent Core or model packs can update without always reflashing the whole OS.

---

## 9. Process & workflow

**Sprint** — Time-boxed delivery chunk with an epic and retrospective. In AuraOS: numbered Sprint 1…N with Confluence retros under the Development Plan.

**Epic** — Large Jira item grouping stories (e.g. SCRUM-33 for Sprint 7). Links implementation PRs and acceptance themes.

**Story / ticket (SCRUM-N)** — Implementable unit of work under an epic.

**devel** — Active integration branch for ongoing sprint work. Feature PRs typically target `devel`.

**master** — Release branch; sprint work lands via release PRs from `devel` after merge.

**PR (Pull Request)** — Proposed git change set for review/merge (GitHub). Feature PRs → `devel`; release PRs `devel` → `master`.

**Retro / retrospective** — Post-sprint write-up of what landed, what slipped, and honesty notes (stubs vs drivers). Child pages under the Dev Plan in Confluence.

**CI** — Continuous Integration checks on PRs (build/test). Keep docs and scripts green without committing local junk (e.g. generated `_adf_*` scratch).

**Acceptance** — Observable "done" signal for a milestone. Example: QEMU serial reaches `sched: idle`; GUI path shows ramfb smoke plus serial `ramfb smoke ok`.

---

## Quick cross-links (prerequisites)

| If you see… | Learn first… |
|-------------|--------------|
| SVC / syscall | EL0, EL1, VBAR |
| Preempt | IRQ, GIC, CNTP, PPI |
| Guest Agent Core | EL0, mailbox IPC, fail-closed, init |
| ramfb smoke | fw_cfg, DMA, framebuffer, QEMU GTK/SDL |
| OTA A/B | manifest, signed, slots, stub vs apply |
| Tier A/B/C | agentic, cloud vs on-device agent, QEMU virt |

---

_ASCII-friendly source of truth in-repo. Keep Confluence in sync when terms or honesty status change._
