# AuraOS Learning Path — Prerequisites to Sprint 11

A mentor-style study guide for someone who knows little about Rust or OS development, mapped to what AuraOS actually built from Foundation through Sprint 11.

**Interactive track (planned course):** prefer **[learning-course.md](learning-course.md)** if you want a **week-by-week curriculum** with free Watch/Read links, Do exercises (rustlings/QEMU), AuraOS labs, self-check quizzes, and a progress checklist (~24 weeks, Blocks 0–6 + capstone). This learning-path doc remains the **sprint-mapped reference** (what each stage built + glossary labs). Confluence: [AuraOS Learning Course — Interactive free curriculum](https://auramislab.atlassian.net/wiki/spaces/AuraOS/pages/557199).

**Goal:** build the skills to read, explain, and eventually rebuild AuraOS features for Sprints 1–11 **without leaning on AI as a crutch**. AI can quiz you; your primary textbook is this repo’s code plus the glossary.

**Honesty up front:** this is **months of deliberate study**, not a weekend crash course. A motivated beginner studying part-time might need **3–9+ months** before the later sprints feel comfortable. That is normal. OS work rewards patience more than speed.

| Band | Rough time (part-time) | What “done” feels like |
|------|------------------------|-------------------------|
| Part A — Prerequisites | 4–10 weeks | You can read Rust, reason about memory, and explain user vs kernel |
| Part B — Systems foundations | 2–6 weeks (overlap OK) | You can sketch boot, MMU, IRQ, IPC, and OTA trust at a whiteboard |
| Part C — Foundation + S1–S4 | 6–14 weeks | You can narrate bring-up through Agent Core on EL0 |
| Part C — S5–S8 | 6–12 weeks | Display, storage, spawn/waitpid, and OTA apply paths make sense |
| Part C — S9–S11 | 4–10 weeks | Soft crypto, VB stubs, custody API, and Pi research honesty click |

Use the **lower** end if you already code and study full-time; the **upper** end if Rust and systems are both new. Skip ahead only when a checkpoint is truly easy — pretending you understand traps and ownership will hurt later.

---

## How to use this path

1. **Study Part A** until the Rust and mental-model checkpoints feel honest.
2. **Skim Part B** in parallel (or right before Foundation labs). You do not need every OS textbook chapter — you need working intuition.
3. **Walk Part C in order** (Foundation → Sprint 11). Each module has: what AuraOS built, concepts (glossary-linked), Rust/systems skills, a repo lab, and a checkpoint.
4. **Treat the glossary as a dictionary, not optional reading.** In-repo: [glossary.md](glossary.md). On Confluence: [AuraOS Glossary — OS & project vocabulary](https://auramislab.atlassian.net/wiki/spaces/AuraOS/pages/557057).
5. **Read code aloud or write notes.** If you cannot explain a file to a rubber duck, you have not finished the lab.
6. **Run the guest** when you can: `scripts/run-qemu.ps1` / `scripts/run-qemu-gui.ps1` and compare serial to [expected-qemu-serial.txt](expected-qemu-serial.txt).

**Confluence twin:** [AuraOS Learning Path — Prerequisites to Sprint 11](https://auramislab.atlassian.net/wiki/spaces/AuraOS/pages/557162)

Companion project docs: [architecture.md](architecture.md) · [agent-core.md](agent-core.md) · [hardware.md](hardware.md) · [hardware-port-pi5.md](hardware-port-pi5.md) · [updates-4y.md](updates-4y.md) · [verified-boot.md](verified-boot.md) · Dev Plan: [AuraOS Development Plan](https://auramislab.atlassian.net/wiki/spaces/AuraOS/pages/295074).

---

## Part A — Prerequisites (before any AuraOS sprint)

Do these **before** treating Foundation as a coding target. Order matters.

### 1. How computers execute programs

**Why:** Every AuraOS lab assumes a [CPU](glossary.md#2-machine--emulation) fetching instructions, registers holding state, and [RAM](glossary.md#mmu-memory-management-unit) as addressable bytes.

**Learn:** fetch–decode–execute; registers vs memory; what a “program counter” is.

**Resource type:** concept / short systems intro (CSAPP-style chapters or any clear “how programs run” lecture). Official books preferred over random blogs.

### 2. Binary, hex, and a pointers/memory mental model

**Why:** VirtIO queues, MMIO bases (`0x0900_0000`), syscall args, and trap frames are all numbers with meaning.

**Learn:** binary ↔ hex; bytes vs words; “a pointer is an address”; stack grows, heap is freer; null and dangling are bugs, not mysteries.

**Resource type:** concept + practice (decode a few hex dumps by hand).

### 3. Calling conventions and the stack (high level)

**Why:** AuraOS syscalls follow AAPCS64: `x8` = syscall number, args in `x0…`, return in `x0`, then `svc #0` ([SVC](glossary.md#svc-supervisor-call), [Syscall](glossary.md#syscall)).

**Learn:** what a stack frame is for; caller/callee-saved intuition; why traps must save registers ([TrapFrame](glossary.md#trapframe)).

**Resource type:** concept (AAPCS64 overview at high level — not memorizing every register).

### 4. Privilege levels (user vs kernel)

**Why:** Guests run at [EL0](glossary.md#el0); the kernel at [EL1](glossary.md#el1). Crossing that boundary is the whole OS game.

**Learn:** least privilege; why apps cannot poke device MMIO freely; what “supervisor” means.

**Resource type:** concept ([Exception level (EL)](glossary.md#exception-level-el)).

### 5. Interrupts vs polling (conceptually)

**Why:** Early VirtIO RX was [polled I/O](glossary.md#polled-io); timer [preemption](glossary.md#preempt--preemption) and later console RX use [IRQ](glossary.md#irq) via the [GIC](glossary.md#gic-generic-interrupt-controller).

**Learn:** “ask repeatedly” vs “device taps you on the shoulder”; latency vs complexity tradeoff.

**Resource type:** concept.

### 6. What an OS kernel does

**Why:** AuraOS is a small [research / teaching kernel](glossary.md#research-os--teaching-kernel), not Android. You need the job description: boot, mediate hardware, schedule processes, enforce isolation, provide syscalls/IPC.

**Learn:** kernel vs userspace; drivers; processes; “policy in userspace, mechanism in kernel” as a design instinct ([Agent Core](glossary.md#agent-core) lives in userspace on purpose).

**Resource type:** concept + skim of [architecture.md](architecture.md).

### 7. Git + terminal basics

**Why:** Day-to-day work is `git`, PowerShell/bash, `cargo`, and QEMU scripts — not a GUI IDE alone.

**Learn:** clone/branch/commit/diff/PR; `cd`, pipes, reading logs; running scripts under `scripts/`.

**Resource type:** official Git book basics + your shell’s help.

### 8. Rust path (non-negotiable depth)

**Why:** Kernel and guests are Rust (`no_std` in the kernel). Ownership bugs become security bugs at EL1.

**Suggested path:**

1. [The Rust Book](https://doc.rust-lang.org/book/) chapters **1–11** (or [Rustlings](https://github.com/rust-lang/rustlings) through ownership exercises) — **do the exercises**, do not only read.
2. **Ownership, borrowing, and lifetimes** until you can explain *why* the borrow checker complains without googling the error first.
3. Later (with Foundation labs): `no_std`, `unsafe`, raw pointers, `core::`, and “why we use `unsafe` at MMIO” — only after safe Rust feels natural.

**Resource type:** official book / official exercises. Defer advanced crates until you need them.

### 9. Optional: tiny ARM64 assembly literacy

**Why:** Reading `exceptions` / trap entry is easier if `ldr`/`str`, registers `x0`–`x30`, and `svc` / `eret` are not alien.

**Learn:** a handful of instructions; that [eret](glossary.md#eret) returns from an exception; that [VBAR](glossary.md#vbar-vector-base-address-register) points at handlers.

**Resource type:** short aarch64 cheat sheet + Arm Architecture Reference (browse, don’t memorize).

**Checkpoint A:** You can explain, in your own words, how a userspace `write` becomes a kernel action (registers → `svc` → handler → device or buffer → return). You have finished Rust Book ch. 1–11 exercises (or equivalent Rustlings) without feeling lost on ownership.

---

## Part B — Systems / OS foundations (parallel or before Sprint 1)

Study these alongside Foundation or just before the matching sprint. Depth = intuition + vocabulary, not a PhD.

| Topic | Why it matters in AuraOS | Glossary anchors |
|-------|--------------------------|------------------|
| Bare metal vs hosted | Kernel is bare metal ([QEMU](glossary.md#qemu) [virt](glossary.md#virt-qemu-machine)); host demos are Tokio apps on your PC | [Host](glossary.md#host), [Guest](glossary.md#guest) |
| Boot flow | Firmware/QEMU → raw `-kernel` → optional [initrd](glossary.md#initrd-initial-ram-disk) → [userspace](glossary.md#userspace) | [Boot path](glossary.md#boot-path), [FDT / DTB](glossary.md#fdt--dtb-flattened-device-tree) |
| MMU / virtual memory | Early [identity map](glossary.md#identity-map); later TTBR0 free on exit | [MMU](glossary.md#mmu-memory-management-unit), [TTBR](glossary.md#ttbr-translation-table-base-register) |
| Device drivers / MMIO | UART, VirtIO, fw_cfg are address windows | [MMIO](glossary.md#mmio-memory-mapped-io), [PL011 UART](glossary.md#pl011-uart), [VirtIO](glossary.md#virtio) |
| Schedulers / preemption | Cooperative yield + timer [preempt](glossary.md#preempt--preemption) | [Scheduler](glossary.md#scheduler-sched), [CNTP](glossary.md#cntp) |
| IPC | Mailboxes on guest; JSON/TCP on host | [IPC](glossary.md#ipc-inter-process-communication), [Mailbox](glossary.md#mailbox) |
| Filesystems / initrd / cpio | Guests arrive as [cpio (newc)](glossary.md#cpio-newc) in initrd, not a full FS yet | [initrd](glossary.md#initrd-initial-ram-disk), [ELF / ELF64](glossary.md#elf--elf64) |
| Crypto basics | Hash integrity vs signature authenticity; [fail-closed](glossary.md#fail-closed) | [Signed / unsigned](glossary.md#signed--unsigned), [HSM](glossary.md#hsm-hardware-security-module), [Verified boot](glossary.md#verified-boot) |

**Checkpoint B:** You can sketch on one page: boot → kernel init → load init → spawn agent/shell → tool request over IPC → (later) OTA verify fail-closed. Label which pieces are stubs vs real drivers using the honesty vocabulary ([Probe / stub vs driver](glossary.md#probe--stub-vs-driver)).

---

## Part C — Sprint-by-sprint study list

For each stage: learn the concepts, then use the **Suggested lab** as your homework. Links point at [glossary.md](glossary.md); prefer the Confluence glossary when studying from the wiki.

---

### Foundation — Skeleton, UART serial, EL0 + basic IPC

**What AuraOS built**

- Teaching-kernel skeleton on QEMU `virt` aarch64: early [PL011 UART](glossary.md#pl011-uart), heap/frames, [identity MMU](glossary.md#identity-map), [VBAR](glossary.md#vbar-vector-base-address-register) / [SVC](glossary.md#svc-supervisor-call).
- Minimal [EL0](glossary.md#el0) processes and syscall path.
- Basic in-kernel [IPC](glossary.md#ipc-inter-process-communication) (mailbox-style channels).

**Concepts to learn**

- [AuraOS](glossary.md#auraos), [From-zero](glossary.md#from-zero), [Research OS / teaching kernel](glossary.md#research-os--teaching-kernel), [aarch64](glossary.md#aarch64), [EL0](glossary.md#el0) / [EL1](glossary.md#el1), [Exception / trap](glossary.md#exception--trap), [Syscall](glossary.md#syscall), [MMIO](glossary.md#mmio-memory-mapped-io)

**Rust / systems skills**

- `no_std` crate layout; `unsafe` for MMIO volatile access; linking a freestanding binary; reading serial as your debugger.

**Suggested lab**

- Read `kernel/src/main.rs` (bring-up order), `kernel/src/uart.rs`, `kernel/src/exceptions.rs`, `kernel/src/trap.rs`, `kernel/src/syscall.rs`, `kernel/src/ipc.rs`.
- Explain aloud: what prints `AuraOS kernel online`, and how an EL0 `write` reaches UART/console.

**Checkpoint:** You can explain the Foundation boot path and sketch SVC entry → `TrapFrame` → syscall dispatch → return/`eret`.

---

### Sprint 1 — VirtIO console TX/RX

**What AuraOS built**

- [VirtIO-console](glossary.md#virtio-console) over VirtIO-MMIO: guest `SYS_WRITE` / `SYS_READ` path.
- TX + RX queues; early RX often [polled](glossary.md#polled-io); UART kept for early/panic.

**Concepts to learn**

- [VirtIO](glossary.md#virtio), [VirtIO-console](glossary.md#virtio-console), [MMIO](glossary.md#mmio-memory-mapped-io), [Polled I/O](glossary.md#polled-io), [Guest](glossary.md#guest) vs kernel console

**Rust / systems skills**

- Device register layout; descriptor/queue mental model; non-blocking read semantics.

**Suggested lab**

- Read `kernel/src/virtio.rs` (console probe, TX/RX), `kernel/src/console.rs`, guest write/read helpers under `userspace/guest/`.
- Trace one character from guest `SYS_WRITE` to the QEMU serial window.

**Checkpoint:** You can explain VirtIO console vs PL011 roles and sketch a TX descriptor path at a high level.

---

### Sprint 2 — Timer preemption + process table

**What AuraOS built**

- [GICv2](glossary.md#gic-generic-interrupt-controller) + [CNTP](glossary.md#cntp) (~100 Hz) → [preempt](glossary.md#preempt--preemption) running EL0.
- Process table with [PID](glossary.md#pid) and Runnable / Running / Blocked / Exited; [scheduler](glossary.md#scheduler-sched) loop toward `sched: idle`.

**Concepts to learn**

- [IRQ](glossary.md#irq), [GIC](glossary.md#gic-generic-interrupt-controller), [PPI](glossary.md#ppi-private-peripheral-interrupt), [CNTP](glossary.md#cntp), [Preempt / preemption](glossary.md#preempt--preemption), [Scheduler (`sched`)](glossary.md#scheduler-sched), [TrapFrame](glossary.md#trapframe)

**Rust / systems skills**

- Shared IRQ/SVC frame layout; state machines for process slots; cooperative `yield` vs forced preempt.

**Suggested lab**

- Read `kernel/src/gic.rs`, `kernel/src/timer.rs`, `kernel/src/process.rs`, `kernel/src/sched.rs`, `kernel/src/trap.rs`.
- Compare serial to [expected-qemu-serial.txt](expected-qemu-serial.txt) until `sched: idle` makes sense.

**Checkpoint:** You can explain when a process becomes Runnable after a timer IRQ and sketch `exception → TrapAction::Preempt → sched::run`.

---

### Sprint 3 — Initrd guest bootstrap

**What AuraOS built**

- Discover [initrd](glossary.md#initrd-initial-ram-disk) from [FDT](glossary.md#fdt--dtb-flattened-device-tree) `/chosen`; parse [cpio (newc)](glossary.md#cpio-newc); load [ELF64](glossary.md#elf--elf64) guests into process slots.
- Guests packed by `scripts/pack-initrd.ps1` into `build/initrd.cpio`.

**Concepts to learn**

- [Boot path](glossary.md#boot-path), [initrd](glossary.md#initrd-initial-ram-disk), [cpio (newc)](glossary.md#cpio-newc), [FDT / DTB](glossary.md#fdt--dtb-flattened-device-tree), [ELF / ELF64](glossary.md#elf--elf64), [init / PID 1](glossary.md#init--pid-1)

**Rust / systems skills**

- Parsing binary formats carefully; separating “pack on host” from “load in kernel.”

**Suggested lab**

- Read `kernel/src/fdt.rs`, `kernel/src/cpio.rs`, `kernel/src/elf.rs`, `kernel/src/bootinfo.rs`, `scripts/pack-initrd.ps1`.
- Name which guest ELF becomes which early process and how the kernel finds it by name.

**Checkpoint:** You can explain QEMU `-initrd` → FDT bounds → cpio lookup → ELF load → first schedule.

---

### Sprint 4 — Agent Core tool loop on EL0 + mailbox IPC

**What AuraOS built**

- Resident [Agent Core](glossary.md#agent-core) on guest EL0: [tool loop](glossary.md#tool--tool-loop) over [mailbox](glossary.md#mailbox) IPC.
- Shell expects tools such as `help` and `system_status`; [fail-closed](glossary.md#fail-closed) if Agent READY is missing.
- Host path still useful: Tokio demos + JSON IPC (`docs/agent-core.md`).

**Concepts to learn**

- [Agentic (agentic OS)](glossary.md#agentic-agentic-os), [Agent Core](glossary.md#agent-core), [Tool / tool loop](glossary.md#tool--tool-loop), [Mailbox](glossary.md#mailbox), [Fail-closed](glossary.md#fail-closed), [Policy gate](glossary.md#policy-gate)

**Rust / systems skills**

- Protocol design with small fixed messages; separating host JSON IPC (`shared/src/ipc.rs`) from guest mailboxes; reading agent/shell as system services, not “apps.”

**Suggested lab**

- Read `userspace/guest/src/bin/guest_agent.rs`, `userspace/guest/src/bin/guest_shell.rs`, `userspace/guest/src/agent_ipc.rs`, `userspace/guest/src/bin/guest_init.rs`, `docs/agent-core.md`, `shared/src/tools.rs`.
- Walk READY (e.g. `0xA11E`) and one tool request/response on the mailbox channels.

**Checkpoint:** You can explain why init fails closed without Agent Core and sketch shell → mailbox → agent tool → response.

---

### Sprint 5 — ramfb / fw_cfg DMA + agent UI shell; VirtIO-GPU probe

**What AuraOS built**

- [ramfb](glossary.md#ramfb) via [fw_cfg](glossary.md#fw_cfg) **DMA** (not plain DATA stores); smoke fill + serial `ramfb smoke ok`.
- Agent-facing UI surface on the host/guest shell path; [VirtIO-GPU](glossary.md#virtio-gpu) **probe** (full scanout later).

**Concepts to learn**

- [ramfb](glossary.md#ramfb), [fw_cfg](glossary.md#fw_cfg), [DMA](glossary.md#dma-direct-memory-access), [Framebuffer](glossary.md#framebuffer), [Smoke test / smoke fill](glossary.md#smoke-test--smoke-fill), [VirtIO-GPU](glossary.md#virtio-gpu), [Probe / stub vs driver](glossary.md#probe--stub-vs-driver), [GTK / SDL](glossary.md#gtk--sdl)

**Rust / systems skills**

- Configuring devices through firmware interfaces; pixel formats (e.g. XRGB8888); keeping probe honesty in logs.

**Suggested lab**

- Read `kernel/src/display.rs`, VirtIO-GPU bits in `kernel/src/virtio.rs`, `userspace/shell/src/framebuffer.rs`, `scripts/run-qemu-gui.ps1`.
- Explain why fw_cfg DMA matters for ramfb activation.

**Checkpoint:** You can explain ramfb smoke vs VirtIO-GPU probe and how to verify GUI + serial acceptance.

---

### Sprint 6 — Pi checklist, OTA A/B skeleton, host ota-verify

**What AuraOS built**

- Raspberry Pi 5 [bring-up](glossary.md#bring-up) research checklist (`docs/hardware-port-pi5.md`) — stubs, not shipping drivers.
- [OTA](glossary.md#ota-over-the-air) [A/B slots](glossary.md#ab-slots) skeleton + host `aura-ota-verify` rejecting unsigned manifests ([fail-closed](glossary.md#fail-closed)).

**Concepts to learn**

- [Bring-up](glossary.md#bring-up), [BSP](glossary.md#bsp-board-support-package), [OTA](glossary.md#ota-over-the-air), [Manifest](glossary.md#manifest), [A/B slots](glossary.md#ab-slots), [Channel (`os` / `agent` / `models`)](glossary.md#channel-os--agent--models), [Signed / unsigned](glossary.md#signed--unsigned), [4-year support / EOS](glossary.md#4-year-support--eos)

**Rust / systems skills**

- Host tooling crates (`tools/ota-verify`); shared types in `shared/src/ota.rs`; reading product docs without confusing roadmap with done work.

**Suggested lab**

- Read `docs/hardware-port-pi5.md`, `docs/updates-4y.md`, `shared/src/ota.rs`, `tools/ota-verify/`, `ota/` fixtures and slot metadata.
- Verify an unsigned fixture fails and a `dev-signed` path behaves as documented.

**Checkpoint:** You can explain A/B intent, channel split, and why host verify refuses unsigned blobs.

---

### Sprint 7 — VirtIO-blk, console IRQ/GIC, OTA apply stub, waitpid

**What AuraOS built**

- [VirtIO-blk](glossary.md#virtio-blk) against QEMU `ab-slots.img` (e.g. sector 0 `AURAAB` header).
- VirtIO console [IRQ](glossary.md#irq) drain via [GIC](glossary.md#gic-generic-interrupt-controller) ([SPI](glossary.md#spi-shared-peripheral-interrupt)), poll remains fallback.
- On-device [OTA](glossary.md#ota-over-the-air) **apply stub**; richer [waitpid](glossary.md#waitpid) for init lifecycle.

**Concepts to learn**

- [VirtIO-blk](glossary.md#virtio-blk), [SPI](glossary.md#spi-shared-peripheral-interrupt), [Polled I/O](glossary.md#polled-io) vs IRQ, [waitpid](glossary.md#waitpid), [Probe / stub vs driver](glossary.md#probe--stub-vs-driver), [Rollback](glossary.md#rollback)

**Rust / systems skills**

- Block I/O at sector granularity; IRQ handler demux; blocking wait helpers on top of process state.

**Suggested lab**

- Read VirtIO-blk + `enable_irqs` / `handle_irq` in `kernel/src/virtio.rs`, `kernel/src/ota.rs`, `waitpid` paths in `kernel/src/process.rs` / `kernel/src/syscall.rs`, guest init wait usage.
- Contrast “stub apply” logs with a real slot write (not claimed yet in S7).

**Checkpoint:** You can explain blk header discovery, console IRQ drain vs poll, and how waitpid supports fail-closed init.

---

### Sprint 8 — SYS_SPAWN, real A/B slot write, sha256-dev, VirtIO-GPU scanout

**What AuraOS built**

- Init-owned [SYS_SPAWN](glossary.md#spawn): kernel boots [init](glossary.md#init--pid-1); init loads agent/shell from initrd.
- Real A/B slot write path on VirtIO-blk (still not production OTA crypto).
- `sha256-dev` digests for development verification; [VirtIO-GPU](glossary.md#virtio-gpu) [scanout](glossary.md#scanout) path (GUI default may still prefer ramfb).

**Concepts to learn**

- [spawn](glossary.md#spawn), [init / PID 1](glossary.md#init--pid-1), [A/B slots](glossary.md#ab-slots), [Scanout](glossary.md#scanout), [VirtIO-GPU](glossary.md#virtio-gpu), [Signed / unsigned](glossary.md#signed--unsigned) (dev digests ≠ production signatures)

**Rust / systems skills**

- Privilege of spawn (init-only); hashing APIs for manifests/payloads; GPU control queue + flush mental model.

**Suggested lab**

- Read `SYS_SPAWN` in `kernel/src/syscall.rs` / `process.rs`, spawn usage in `guest_init.rs`, blk write helpers in `virtio.rs`, GPU scanout (`init_gpu_scanout`), hash usage near OTA/shared code.
- Narrate boot: kernel → init only → spawn agent → spawn shell.

**Checkpoint:** You can sketch init-owned spawn and explain sha256-dev as integrity-for-dev, not HSM-backed trust.

---

### Sprint 9 — Soft ed25519 on host, VB stub, TTBR0 free on exit

**What AuraOS built**

- Host soft [ed25519](glossary.md#signed--unsigned) verify path (`shared::trust::SoftEd25519`).
- [Verified boot](glossary.md#verified-boot) **stub** + docs (`docs/verified-boot.md`) — not production silicon trust.
- Free/destroy user address space ([TTBR](glossary.md#ttbr-translation-table-base-register) / TTBR0) on process exit for hygiene.

**Concepts to learn**

- Hash vs signature; [Verified boot](glossary.md#verified-boot), [HSM](glossary.md#hsm-hardware-security-module) (deferred), [TTBR](glossary.md#ttbr-translation-table-base-register), [Identity map](glossary.md#identity-map)

**Rust / systems skills**

- `TrustBackend`-style APIs; `no_std`-friendly crypto boundaries on host first; page-table lifecycle.

**Suggested lab**

- Read `shared/src/trust.rs`, `docs/verified-boot.md`, `kernel/src/vb.rs`, `kernel/src/vm.rs` (`destroy_address_space` / TTBR0 switch).
- Explain what the VB stub allows/denies today versus a real chain of trust.

**Checkpoint:** You can explain soft ed25519 vs HSM, what the VB stub guarantees, and why freeing TTBR0 on exit matters.

---

### Sprint 10 — Kernel soft ed25519 wired to OTA, HSM-ready trust API, Pi prep

**What AuraOS built**

- Kernel `no_std` soft ed25519 wired into on-device OTA/VB paths (`kernel/src/ota_crypto.rs` and friends).
- HSM-ready [TrustBackend](glossary.md#hsm-hardware-security-module) API extension — helpers + docs; **not** a real HSM.
- Pi UART/GIC/DT bring-up prep: checklist and honest stubs (`kernel/src/board_pi5.rs`, docs).

**Concepts to learn**

- [OTA](glossary.md#ota-over-the-air), [Verified boot](glossary.md#verified-boot), [HSM](glossary.md#hsm-hardware-security-module), [Bring-up](glossary.md#bring-up), [PL011 UART](glossary.md#pl011-uart), [GIC](glossary.md#gic-generic-interrupt-controller)

**Rust / systems skills**

- Porting verify into `no_std`; trait objects / backend selection without claiming production readiness; board feature flags that stay off until real silicon work.

**Suggested lab**

- Read `kernel/src/ota_crypto.rs`, OTA apply glue in `kernel/src/ota.rs`, `shared/src/trust.rs` backend kinds, `kernel/src/board_pi5.rs`, Sprint honesty notes in Dev Plan retros.
- Point to one log/string that proves “stub/research, not driver.”

**Checkpoint:** You can explain the kernel soft-verify path and why `HsmDeferred` must fail closed for production claims.

---

### Sprint 11 — KeyHandle custody scaffolding, VB stage checks, Pi UART/GIC research constants

**What AuraOS built**

- [KeyHandle](glossary.md#hsm-hardware-security-module) / `CustodyKind` custody scaffolding (`verify_with_handle`); soft-dev handles work; HSM slot handles stay deferred/fail-closed.
- VB **per-stage** stub checks (boot-adjacent stages) with clearer serial honesty.
- Pi UART/GIC research constants (`EarlyConsoleMap`, redistributor/DT hints) — still not claiming silicon success.

**Concepts to learn**

- Key custody vs raw key bytes; [Fail-closed](glossary.md#fail-closed), [Verified boot](glossary.md#verified-boot), [Probe / stub vs driver](glossary.md#probe--stub-vs-driver), [Bring-up](glossary.md#bring-up)

**Rust / systems skills**

- API design that allows a future PKCS#11/cloud HSM without fake keys today; staged gates; board constant tables as research, not drivers.

**Suggested lab**

- Read `KeyHandle` / `CustodyKind` in `shared/src/trust.rs`, `kernel/src/vb.rs` stage checks, `kernel/src/board_pi5.rs` (`EarlyConsoleMap`, status lines), [expected-qemu-serial.txt](expected-qemu-serial.txt) for Sprint 11 strings.
- Write a short note: “What would have to be true before we delete the word *stub*?”

**Checkpoint:** You can explain custody handles, VB stage stub behavior, and list what is still deferred (real HSM, silicon VB, Pi UART/GIC drivers).

---

## Part D — Practice order / milestones

Suggested sequence (adjust pace, not order):

1. **Rust fluency** — Book ch. 1–11 / Rustlings; ownership until boring.
2. **Tiny no_std blink/UART toy** (any aarch64 QEMU virt tutorial) — prove you can print without an OS.
3. **Read AuraOS UART + Foundation** — map your toy onto `kernel/src/uart.rs` and bring-up.
4. **Recreate one subsystem at a time** (notes or a personal branch — do not spam production PRs):
   - VirtIO console TX only → add RX poll → timer preempt → initrd load → mailbox agent → ramfb smoke → blk read → spawn/waitpid → soft verify → custody API reading.
5. **Milestone demos you can show yourself:**
   - Serial reaches `sched: idle` and you can narrate every major log line.
   - GUI path: ramfb smoke + expected serial.
   - Host: unsigned OTA rejected; soft ed25519 path explained.
   - You can teach Foundation–S4 to another beginner using only glossary + code.

When stuck: re-read the glossary entry, then the file, then the matching Dev Plan retro. Retros are honesty records — use them.

---

## Part E — Resources (curated)

### Official / well-known

- [The Rust Book](https://doc.rust-lang.org/book/) — primary language path.
- [Rustlings](https://github.com/rust-lang/rustlings) — drills for ownership and syntax.
- [The Embedded Rust Book](https://docs.rust-embedded.org/book/) — `no_std`, peripherals, mindset (concepts transfer to a teaching kernel).
- [Writing an OS in Rust](https://os.phil-opp.com/) (Phil Opp) — **x86_64**, but interrupt/page-table *ideas* transfer; do not copy PC-specific details into AuraOS aarch64 code blindly.
- [OSDev Wiki](https://wiki.osdev.org/) — use carefully; prefer pages that match **aarch64** / VirtIO / GIC topics; verify against Arm docs and this repo.
- Arm Architecture Reference Manual (A-profile) — browse exception levels, VBAR, timers; treat as reference, not a novel.
- Bare-metal Rust + aarch64 QEMU virt tutorials (community) — for the “UART toy” milestone; always cross-check addresses with AuraOS (`0x0900_0000` UART on virt).

### Project docs (primary textbook)

| Doc | Use it for |
|-----|------------|
| [glossary.md](glossary.md) / [Confluence glossary](https://auramislab.atlassian.net/wiki/spaces/AuraOS/pages/557057) | Vocabulary — link every confusing word here first |
| [architecture.md](architecture.md) | Layers, bring-up, syscalls, trap path |
| [agent-core.md](agent-core.md) | Agent as system service |
| [expected-qemu-serial.txt](expected-qemu-serial.txt) | Acceptance / lab oracle |
| [hardware-port-pi5.md](hardware-port-pi5.md) | Pi research honesty |
| [updates-4y.md](updates-4y.md) | OTA product promise |
| [verified-boot.md](verified-boot.md) | VB stub vs production |
| Sprint retros under the [Dev Plan](https://auramislab.atlassian.net/wiki/spaces/AuraOS/pages/295074) | What landed vs what slipped |

### Study habits that work

- **Code first, blog second.** AuraOS is small enough that `kernel/src/*.rs` is readable end-to-end.
- **Write one page of notes per sprint module** in your own words; if you only paste AI summaries, you skipped the learning.
- **Say “stub” when the project says stub.** Shipping confidence comes from honesty, not vibes.

---

## Coverage checklist

| Stage | Covered in Part C |
|-------|-------------------|
| Foundation | Yes |
| Sprint 1 | Yes |
| Sprint 2 | Yes |
| Sprint 3 | Yes |
| Sprint 4 | Yes |
| Sprint 5 | Yes |
| Sprint 6 | Yes |
| Sprint 7 | Yes |
| Sprint 8 | Yes |
| Sprint 9 | Yes |
| Sprint 10 | Yes |
| Sprint 11 | Yes |

Glossary: prefer [docs/glossary.md](glossary.md) (~80+ defined terms) and keep Confluence page 557057 in sync when terms change.

---

_You do not need to become an OS expert before cloning the repo. You do need to respect the depth: one honest lab at a time, glossary open, code as the textbook._
