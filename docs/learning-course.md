# AuraOS Learning Course — Interactive free curriculum

A **week-by-week planned course** for learning Rust and OS ideas well enough to read, explain, and eventually extend AuraOS — using **only free, trusted resources** plus this repo.

**Who this is for:** beginners who want a schedule with homework, quizzes, and repo labs — not just a reading list.

**Companion reference track:** [learning-path.md](learning-path.md) (sprint-mapped study guide).  
**Glossary:** [glossary.md](glossary.md) · [Confluence glossary](https://auramislab.atlassian.net/wiki/spaces/AuraOS/pages/557057)  
**Dev Plan:** [AuraOS Development Plan](https://auramislab.atlassian.net/wiki/spaces/AuraOS/pages/295074)

**Honesty:** part-time (≈6–10 h/week) this is roughly **5–6 months**. Full-time or prior systems experience can compress Blocks 1–2. Do **not** skip ownership or privilege quizzes — later weeks assume them.

| Block | Focus | Weeks | Hours (est.) |
|-------|--------|-------|--------------|
| 0 | Tooling + first QEMU boot | 1 | 4–8 |
| 1 | Rust fluency | 4 | 24–40 |
| 2 | Systems mental model | 3 | 12–21 |
| 3 | Bare-metal Rust → AuraOS UART | 3 | 15–24 |
| 4 | Foundation → Sprint 4 | 5 | 25–40 |
| 5 | Sprints 5–8 | 4 | 20–32 |
| 6 | Sprints 9–11 | 3 | 15–24 |
| Capstone | Tiny change without AI writing code | 1 | 4–10 |
| **Total** | | **~24 weeks** | **~120–200 h** |

**How each week works**

1. **Objectives** — what you can *do* by Friday.
2. **Watch / Read** — free URLs only (allowlist below).
3. **Do** — interactive drills (rustlings, notes, explain-aloud).
4. **AuraOS lab** — open specific files in this repo.
5. **Self-check quiz** — try first; answers live in [Answers appendix](#answers-appendix).
6. **Estimated hours** — realistic part-time budget.

**Rule:** AI may *quiz* you. It must not *do* rustlings, labs, or the capstone for you.

---

## Free resource allowlist

Use these as primary sources. Prefer them over random blogs and paid courses.

| Resource | URL |
|----------|-----|
| The Rust Book | https://doc.rust-lang.org/book/ |
| Rust by Example | https://doc.rust-lang.org/rust-by-example/ |
| Rustlings | https://github.com/rust-lang/rustlings |
| The Rustonomicon (later) | https://doc.rust-lang.org/nomicon/ |
| The Embedded Rust Book | https://docs.rust-embedded.org/book/ |
| OSTEP | https://pages.cs.wisc.edu/~remzi/OSTEP/ |
| Writing an OS in Rust (Phil Opp) — **x86 concepts** | https://os.phil-opp.com/ |
| OSDev Wiki | https://wiki.osdev.org/Main_Page |
| VirtIO 1.2 spec | https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html |
| QEMU docs | https://www.qemu.org/docs/master/ |
| Arm Learn the Architecture — AArch64 exception model | https://developer.arm.com/documentation/102412/latest |
| Arm Learn the Architecture — AArch64 memory management | https://developer.arm.com/documentation/101811/latest |
| Git Book (basics) | https://git-scm.com/book/en/v2 |
| Project textbook | [glossary.md](glossary.md), [learning-path.md](learning-path.md), [architecture.md](architecture.md), [expected-qemu-serial.txt](expected-qemu-serial.txt) |

Optional free video (if you want lectures): [MIT 6.828 / OCW OS materials](https://ocw.mit.edu/) — search current free offerings; **text above is enough** if URLs move.

---

## Progress checklist

Copy this into a notebook or tick in-place.

### Block 0 — Setup
- [ ] Week 0 — Tools + first `run-qemu.ps1`

### Block 1 — Rust fluency
- [ ] Week 1 — Getting started + ownership intro
- [ ] Week 2 — Ownership deep dive
- [ ] Week 3 — Structs, enums, modules, collections
- [ ] Week 4 — Errors, generics, traits, tests

### Block 2 — Systems mental model
- [ ] Week 5 — Processes, privilege, direct execution
- [ ] Week 6 — Address spaces & VM intuition
- [ ] Week 7 — Concurrency intro + interrupts

### Block 3 — Bare metal Rust
- [ ] Week 8 — Embedded / `no_std` mindset
- [ ] Week 9 — OS ideas (Phil Opp) + aarch64 literacy
- [ ] Week 10 — AuraOS UART / console bridge

### Block 4 — Foundation → Sprint 4
- [ ] Week 11 — Foundation (UART, EL, traps, IPC)
- [ ] Week 12 — Sprint 1 (VirtIO console)
- [ ] Week 13 — Sprint 2 (timer + scheduler)
- [ ] Week 14 — Sprint 3 (initrd / ELF)
- [ ] Week 15 — Sprint 4 (Agent Core + mailbox)

### Block 5 — Sprints 5–8
- [ ] Week 16 — Sprint 5 (ramfb / GPU probe)
- [ ] Week 17 — Sprint 6 (OTA skeleton + Pi honesty)
- [ ] Week 18 — Sprint 7 (blk, IRQ, waitpid)
- [ ] Week 19 — Sprint 8 (spawn, slot write, scanout)

### Block 6 — Sprints 9–11
- [ ] Week 20 — Sprint 9 (soft crypto, VB stub, TTBR0)
- [ ] Week 21 — Sprint 10 (kernel ed25519, HSM-ready API)
- [ ] Week 22 — Sprint 11 (custody, VB stages, Pi constants)

### Capstone
- [ ] Week 23 — Tiny change + PR-style write-up (no AI-authored code)

---

# Block 0 — Setup (1 week)

## Week 0 — Tools + first AuraOS boot

**Estimated hours:** 4–8

### Objectives
- Install Git, a Rust toolchain, and QEMU for aarch64.
- Clone AuraOS and open it in VS Code or Cursor.
- Boot the guest once with `scripts/run-qemu.ps1` and recognize serial output.

### Watch / Read
- [Git Book — Getting Started](https://git-scm.com/book/en/v2/Getting-Started-About-Version-Control) (ch. 1–2 skim)
- [Rust Book — Installation](https://doc.rust-lang.org/book/ch01-01-installation.html)
- [QEMU documentation](https://www.qemu.org/docs/master/) — skim “Invocation” / system emulation overview
- Repo README / scripts comments for Windows PowerShell QEMU flow

### Do
1. Install `rustup`, then `rustc --version` and `cargo --version`.
2. Install QEMU (Windows: package manager or official builds) so `qemu-system-aarch64` is on `PATH`.
3. Clone this repo; open the workspace root in your editor.
4. Run `scripts/run-qemu.ps1` once; save a screenshot or paste of the last ~20 serial lines.
5. Explain aloud (60 seconds): host vs guest, and why we use QEMU `virt`.

### AuraOS lab
- Open: `scripts/run-qemu.ps1`, `docs/expected-qemu-serial.txt`, `docs/glossary.md` (Host / Guest / QEMU / virt entries)
- Skim: `docs/architecture.md` (layers only — do not deep-dive yet)

### Self-check quiz
1. What does “guest” mean in AuraOS docs?
2. Name one reason we develop on QEMU `virt` before real Pi silicon.
3. Where do you look to know if a serial line is “expected” for acceptance?
4. What command/script boots the teaching kernel for serial smoke?

→ [Answers — Week 0](#week-0)

---

# Block 1 — Rust fluency (4 weeks; stretch to 5 if ownership fights you)

## Week 1 — Getting started + ownership intro

**Estimated hours:** 6–10

### Objectives
- Write and run small Rust programs with Cargo.
- Explain ownership at a beginner level (who frees memory, and when).
- Complete early Rustlings exercises without copying answers blindly.

### Watch / Read
- [Rust Book ch. 1–3](https://doc.rust-lang.org/book/ch01-00-getting-started.html) (Getting Started, Guessing Game, Common Concepts)
- [Rust Book ch. 4.1](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) (What is Ownership?)
- Optional parallel: [Rust by Example — Hello World / Primitives](https://doc.rust-lang.org/rust-by-example/)

### Do
1. Clone/install [Rustlings](https://github.com/rust-lang/rustlings); finish intro + variables + functions chapters.
2. In a scratch crate, write a function that takes ownership of a `String` and one that borrows `&str`; note which compiles.
3. One page of notes: “stack vs heap in Rust, in my own words.”

### AuraOS lab
- Open: workspace `Cargo.toml` / crate layout (kernel vs userspace vs shared) — just map names, do not read unsafe yet.
- Glossary: skim “Research OS / teaching kernel.”

### Self-check quiz
1. What happens to a `String` after it is moved into a function?
2. Why does Rust prefer `&str` for read-only string views?
3. What does `cargo check` buy you vs only running `cargo run`?
4. Name the three crates/areas you noticed in this monorepo.

→ [Answers — Week 1](#week-1)

---

## Week 2 — Ownership deep dive

**Estimated hours:** 6–10

### Objectives
- Predict borrow-checker errors before compiling.
- Use mutable and immutable references correctly; avoid dangling refs.
- Finish Rustlings ownership / borrowing / move exercises.

### Watch / Read
- [Rust Book ch. 4 (full)](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Rust by Example — Ownership](https://doc.rust-lang.org/rust-by-example/scope/move.html) and [Borrowing](https://doc.rust-lang.org/rust-by-example/scope/borrow.html)

### Do
1. Rustlings: ownership + borrowing + moves until green.
2. Deliberately write *broken* snippets (two mutable borrows; use-after-move); fix them yourself.
3. Explain aloud: “Why AuraOS cannot casually share a mutable UART driver without rules.”

### AuraOS lab
- Open: `kernel/src/uart.rs` — find where output is performed; note `unsafe` presence without diving into MMIO yet.
- Write 5 lines: “What ownership problem would a global mutable UART create?”

### Self-check quiz
1. Can you have two `&mut T` to the same value at once? Why?
2. What is the difference between `Copy` types and moved types?
3. What does a lifetime annotation *communicate* (not “magic syntax”)?
4. Why is “just clone everything” a bad habit for kernel code?

→ [Answers — Week 2](#week-2)

---

## Week 3 — Structs, enums, modules, collections

**Estimated hours:** 6–10

### Objectives
- Model state with structs and enums (`Option` / `Result` intuition).
- Organize code with modules; use `Vec` / `HashMap` in host-side practice.
- Progress Rustlings through structs, enums, modules, collections.

### Watch / Read
- [Rust Book ch. 5–8](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [Rust by Example — Structs / Enums / Collections](https://doc.rust-lang.org/rust-by-example/)

### Do
1. Rustlings: structs, enums, modules, collections chapters.
2. Model a tiny “process slot” enum: `Runnable | Running | Blocked | Exited` in a scratch file (names only — no kernel yet).
3. Notes: when to use `enum` vs `struct` for OS state machines.

### AuraOS lab
- Open: `kernel/src/process.rs` (skim process states / table shape only).
- Glossary: PID, Scheduler.

### Self-check quiz
1. How does `Option<T>` force you to handle “missing”?
2. What does `mod` / `use` buy a large crate like the kernel?
3. Why might a process table be a fixed array rather than a growable `Vec` in early kernels?
4. Name one AuraOS process state you spotted (or expect).

→ [Answers — Week 3](#week-3)

---

## Week 4 — Errors, generics, traits, tests

**Estimated hours:** 6–10

### Objectives
- Use `Result` idiomatically; write simple generics and traits.
- Run `cargo test` on a tiny crate; prefer tests over print debugging for pure logic.
- Finish Rustlings through error handling / generics / traits / tests (Book ≈ ch. 9–11).

### Watch / Read
- [Rust Book ch. 9–11](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- Optional: [Rust by Example — Error handling](https://doc.rust-lang.org/rust-by-example/error.html)

### Do
1. Complete Rustlings through the Book ch. 1–11 equivalent set.
2. Checkpoint A rehearsal: explain how a userspace `write` *might* become a kernel action (registers → syscall → return) at whiteboard level — gaps OK; revisit in Block 2–4.
3. One-page “Rust fluency certificate”: list 5 things you still find hard (honest list).

### AuraOS lab
- Open: `shared/src/` (skim module names — tools, ota, trust).
- Skim: [learning-path.md](learning-path.md) Part A checkpoint.

### Self-check quiz
1. When is `panic!` acceptable vs returning `Result`?
2. What problem do traits solve that generics alone do not?
3. Why test pure parsing (cpio/ELF later) more than MMIO poke loops?
4. Have you finished Book ch. 1–11 *exercises* (or Rustlings equivalent)? Yes/No — be honest.

→ [Answers — Week 4](#week-4)

---

# Block 2 — Systems mental model (3 weeks)

## Week 5 — Processes, privilege, direct execution

**Estimated hours:** 4–7

### Objectives
- Explain process abstraction and limited direct execution.
- Define user vs kernel (EL0 vs EL1) in AuraOS vocabulary.
- Sketch “syscall = controlled privilege transition.”

### Watch / Read
- OSTEP (free PDFs from [OSTEP home](https://pages.cs.wisc.edu/~remzi/OSTEP/)):
  - [Introduction](https://pages.cs.wisc.edu/~remzi/OSTEP/intro.pdf)
  - [Processes](https://pages.cs.wisc.edu/~remzi/OSTEP/cpu-intro.pdf)
  - [Direct Execution](https://pages.cs.wisc.edu/~remzi/OSTEP/cpu-mechanisms.pdf)
- Glossary: EL0, EL1, Exception level, Syscall, SVC

### Do
1. One-page notes: limited direct execution + traps/syscalls.
2. Explain aloud: why apps must not poke UART MMIO at `0x0900_0000` freely.
3. Draw: user code → `svc` → kernel → `eret` (labels only).

### AuraOS lab
- Open: `docs/glossary.md` (EL / Syscall / TrapFrame), `docs/architecture.md` (privilege / syscall section if present)
- Peek: `kernel/src/syscall.rs` (names of syscalls only)

### Self-check quiz
1. What problem does “limited direct execution” solve?
2. What is EL0 vs EL1 in AuraOS?
3. Why is a syscall not just “a function call”?
4. Name one register role in AAPCS64 syscall convention (AuraOS).

→ [Answers — Week 5](#week-5)

---

## Week 6 — Address spaces & VM intuition

**Estimated hours:** 4–7

### Objectives
- Explain address spaces and why every process believes it owns “the” memory map.
- Define MMU / identity map at teaching-kernel depth (not full Arm ARM).
- Connect “pointer = address” to later VirtIO/descriptor work.

### Watch / Read
- OSTEP: [Address Spaces](https://pages.cs.wisc.edu/~remzi/OSTEP/vm-intro.pdf), [Address Translation](https://pages.cs.wisc.edu/~remzi/OSTEP/vm-mechanism.pdf) (skim paging intro)
- Arm (free): [AArch64 memory management](https://developer.arm.com/documentation/101811/latest) — browse, do not memorize
- Glossary: MMU, Identity map, TTBR

### Do
1. Notes: virtual vs physical; what “identity map” means for early bring-up.
2. Hex practice: convert `0x0900_0000` related sizes; explain “MMIO window.”
3. Explain aloud: why freeing TTBR0 on exit is hygiene (preview Sprint 9).

### AuraOS lab
- Open: `kernel/src/vm.rs` (skim function names), glossary MMU/TTBR
- Architecture doc: boot / memory map notes

### Self-check quiz
1. What is an address space?
2. What does an early identity map buy a teaching kernel?
3. What is MMIO in one sentence?
4. What does TTBR roughly point at?

→ [Answers — Week 6](#week-6)

---

## Week 7 — Concurrency intro + interrupts

**Estimated hours:** 4–7

### Objectives
- Contrast polling vs interrupts; explain preemption at a high level.
- Skim OSTEP concurrency dialogue/intro enough to fear shared mutable state.
- Preview GIC + timer as “hardware scheduler helpers.”

### Watch / Read
- OSTEP: [CPU Scheduling](https://pages.cs.wisc.edu/~remzi/OSTEP/cpu-sched.pdf) (concepts), [Concurrency and Threads](https://pages.cs.wisc.edu/~remzi/OSTEP/threads-intro.pdf) (intro only)
- Glossary: IRQ, GIC, PPI, Polled I/O, Preempt, CNTP
- Optional: [OSDev Wiki — Interrupts](https://wiki.osdev.org/Interrupts) (cross-check aarch64 later)

### Do
1. Table in notes: Polling vs IRQ — latency, complexity, AuraOS examples (UART early vs timer preempt).
2. Explain aloud: cooperative `yield` vs timer preemption.
3. Checkpoint B rehearsal: sketch boot → kernel → init → agent on one page (stubs labeled later).

### AuraOS lab
- Open: `kernel/src/timer.rs`, `kernel/src/gic.rs` (headers / comments only)
- `docs/expected-qemu-serial.txt` — find `sched: idle` line

### Self-check quiz
1. Give one AuraOS example of polling and one of IRQ-driven work.
2. What does preemption mean for a running EL0 process?
3. Why is shared mutable state scary even in a single-core teaching kernel?
4. What log line suggests the scheduler has nothing runnable?

→ [Answers — Week 7](#week-7)

---

# Block 3 — Bare metal Rust (3 weeks)

## Week 8 — Embedded / `no_std` mindset

**Estimated hours:** 5–8

### Objectives
- Explain what `#![no_std]` removes and why kernels care.
- Describe volatile MMIO access as “talking to hardware,” not normal RAM.
- Read Embedded Rust Book early chapters without needing a physical board.

### Watch / Read
- [Embedded Rust Book — Intro + no_std](https://docs.rust-embedded.org/book/)
- [Rustonomicon — Meet Safe and Unsafe](https://doc.rust-lang.org/nomicon/meet-safe-and-unsafe.html) (short; later depth OK)
- Glossary: Bare metal, MMIO, PL011 UART

### Do
1. Notes: `core` vs `std`; what breaks without an allocator / OS.
2. Explain aloud: why `println!` is not available in early kernel bring-up the same way.
3. List three things that must be `unsafe` at the hardware boundary (your list).

### AuraOS lab
- Open: `kernel/src/main.rs` (crate attributes / bring-up order comments)
- Compare: host demos under `userspace/` that *do* use std/Tokio vs guest/kernel

### Self-check quiz
1. What does `no_std` disable?
2. Why is MMIO not “just a pointer write”?
3. Where does early debug output go in AuraOS Foundation?
4. Name one host-side component that is *not* bare metal.

→ [Answers — Week 8](#week-8)

---

## Week 9 — OS ideas (Phil Opp) + aarch64 literacy

**Estimated hours:** 5–8

### Objectives
- Extract *ideas* from Phil Opp (interrupts, pages, VGA-as-debug) without copying x86 details into AuraOS.
- Recognize a handful of AArch64 ideas: `svc`, `eret`, VBAR, registers `x0`–`x30`.
- Know where to look in Arm docs when stuck.

### Watch / Read
- [Writing an OS in Rust — freestanding / VGA / interrupts intro posts](https://os.phil-opp.com/) — read early posts for *ideas*; note **x86_64 ≠ aarch64**
- [Arm AArch64 exception model](https://developer.arm.com/documentation/102412/latest) — browse
- [OSDev Wiki — AArch64](https://wiki.osdev.org/Category:ARM) / bare metal pages — use carefully
- Glossary: VBAR, Exception / trap, eret, aarch64

### Do
1. Two-column notes: “Phil Opp idea” vs “AuraOS aarch64 analogue” (UART serial ≈ VGA text; IDT ≈ VBAR; etc.).
2. Cheat sheet (10 lines): `svc`, `eret`, EL0/EL1, VBAR.
3. Explain aloud: why copying x86 page-table code into this repo would be wrong.

### AuraOS lab
- Open: `kernel/src/exceptions.rs`, `kernel/src/trap.rs` (structure / comments)
- Architecture.md trap path section

### Self-check quiz
1. Name one Phil Opp concept that transfers and one that does not.
2. What does VBAR point to?
3. What does `eret` do?
4. Why is QEMU `virt` UART base not the same as Phil Opp’s VGA buffer address?

→ [Answers — Week 9](#week-9)

---

## Week 10 — AuraOS UART / console bridge

**Estimated hours:** 5–8

### Objectives
- Narrate Foundation early print path from bring-up to PL011.
- Distinguish early UART vs later VirtIO console roles (preview).
- Run QEMU again and map 3 serial lines to code paths.

### Watch / Read
- [QEMU docs](https://www.qemu.org/docs/master/) — virt machine / serial as needed
- [learning-path.md](learning-path.md) — Foundation module
- Glossary: PL011 UART, Console, Bring-up

### Do
1. Trace notes: `main` init order → UART init → first banner string.
2. Explain aloud (2 minutes): “How does `AuraOS kernel online` (or equivalent banner) get to my terminal?”
3. Compare your serial capture to `expected-qemu-serial.txt` — list 3 matches.

### AuraOS lab
- Open: `kernel/src/main.rs`, `kernel/src/uart.rs`, `kernel/src/console.rs`
- Scripts: `scripts/run-qemu.ps1`
- Oracle: `docs/expected-qemu-serial.txt`

### Self-check quiz
1. What device is used for earliest kernel prints?
2. Why keep UART even after VirtIO console exists?
3. Which file owns low-level PL011 access?
4. What is the lab oracle file for serial acceptance?

→ [Answers — Week 10](#week-10)

---

# Block 4 — Map to Foundation–Sprint 4 (5 weeks)

## Week 11 — Foundation (UART, EL, traps, IPC)

**Estimated hours:** 5–8

### Objectives
- Sketch SVC → TrapFrame → syscall dispatch → `eret`.
- Explain minimal EL0 process + basic IPC role.
- Teach Foundation to a rubber duck using only glossary + code.

### Watch / Read
- [learning-path.md — Foundation](learning-path.md)
- [architecture.md](architecture.md)
- Arm exception model (refresh): https://developer.arm.com/documentation/102412/latest

### Do
1. Draw the trap path; label TrapFrame fields you can name.
2. Explain aloud: userspace `write` → kernel action.
3. Notes: what “teaching kernel” promises and does *not* promise.

### AuraOS lab
- `kernel/src/main.rs`, `uart.rs`, `exceptions.rs`, `trap.rs`, `syscall.rs`, `ipc.rs`

### Self-check quiz
1. What is a TrapFrame for?
2. Which exception level runs guest user code?
3. Where does syscall number conventionally arrive (AuraOS/AAPCS64)?
4. What is basic IPC doing for you before Agent Core mailboxes?

→ [Answers — Week 11](#week-11)

---

## Week 12 — Sprint 1 (VirtIO console)

**Estimated hours:** 5–8

### Objectives
- Explain VirtIO-MMIO console TX/RX at a high level.
- Contrast polled RX vs later IRQ drain.
- Trace one character from guest `SYS_WRITE` toward the host serial window.

### Watch / Read
- [VirtIO 1.2 spec](https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html) — console device overview + virtqueues (skim)
- [OSDev Wiki — Virtio](https://wiki.osdev.org/Virtio) (verify against spec + repo)
- learning-path Sprint 1; glossary VirtIO / VirtIO-console / Polled I/O

### Do
1. Notes: descriptor / available / used ring in your own words (imperfect OK).
2. Explain aloud: VirtIO console vs PL011 responsibilities.
3. Optional: highlight queue init in code with editor bookmarks.

### AuraOS lab
- `kernel/src/virtio.rs` (console probe, TX/RX), `kernel/src/console.rs`
- Guests under `userspace/guest/` write/read helpers

### Self-check quiz
1. What transport does AuraOS use for VirtIO on QEMU virt (MMIO vs PCI)?
2. Why was early RX often polled?
3. Does guest `SYS_WRITE` talk to PL011 directly after Sprint 1?
4. Where do you read the device register layout from (prefer official)?

→ [Answers — Week 12](#week-12)

---

## Week 13 — Sprint 2 (timer + scheduler)

**Estimated hours:** 5–8

### Objectives
- Explain GICv2 + CNTP → preempt → scheduler loop.
- Map process states Runnable/Running/Blocked/Exited to code.
- Make `sched: idle` meaningful in your serial log.

### Watch / Read
- OSTEP scheduling refresh: https://pages.cs.wisc.edu/~remzi/OSTEP/cpu-sched.pdf
- learning-path Sprint 2; glossary GIC, CNTP, Preempt, Scheduler
- Optional Arm GIC overview materials on developer.arm.com (GICv2 concepts; AuraOS uses GICv2 on virt)

### Do
1. Sequence diagram (text): IRQ → trap → `TrapAction::Preempt` → `sched::run`.
2. Explain aloud: yield vs preempt.
3. Diff your serial vs expected until idle makes sense.

### AuraOS lab
- `kernel/src/gic.rs`, `timer.rs`, `process.rs`, `sched.rs`, `trap.rs`
- `docs/expected-qemu-serial.txt`

### Self-check quiz
1. About how often does the teaching timer tick (order of magnitude)?
2. What does preemption do to the running EL0 process?
3. When do you see `sched: idle`?
4. Which controller delivers IRQs to the CPU in this stack?

→ [Answers — Week 13](#week-13)

---

## Week 14 — Sprint 3 (initrd / ELF)

**Estimated hours:** 5–8

### Objectives
- Narrate `-initrd` → FDT `/chosen` → cpio → ELF load → schedule.
- Separate host packing (`pack-initrd.ps1`) from kernel loading.
- Name which guest binaries become early processes.

### Watch / Read
- learning-path Sprint 3; glossary initrd, cpio newc, FDT/DTB, ELF64, init/PID1
- [OSDev Wiki — Initrd](https://wiki.osdev.org/Initrd) / ramdisk concepts (verify)
- QEMU docs: `-kernel` / `-initrd` invocation notes

### Do
1. Notes: newc cpio layout at “header + name + data” level.
2. Explain aloud: how the kernel finds a guest by name.
3. Run pack script if needed; know where `build/initrd.cpio` comes from.

### AuraOS lab
- `kernel/src/fdt.rs`, `cpio.rs`, `elf.rs`, `bootinfo.rs`
- `scripts/pack-initrd.ps1`

### Self-check quiz
1. Where does the kernel learn initrd bounds on QEMU virt?
2. What archive format packs guest ELFs?
3. What file format are the guest programs?
4. Who packs the initrd — kernel or host script?

→ [Answers — Week 14](#week-14)

---

## Week 15 — Sprint 4 (Agent Core + mailbox IPC)

**Estimated hours:** 5–8

### Objectives
- Explain Agent Core as a privileged *userspace* service on EL0.
- Walk READY + one tool request/response on mailbox channels.
- State why init fails closed without Agent READY.

### Watch / Read
- [agent-core.md](agent-core.md)
- learning-path Sprint 4; glossary Agent Core, Tool loop, Mailbox, Fail-closed, Policy gate
- Skim host JSON IPC: `shared/src/ipc.rs` vs guest mailboxes

### Do
1. Sequence: shell → mailbox → agent tool → response.
2. Explain aloud: why Agent Core is not “just another app.”
3. Notes: host Tokio demo path vs guest EL0 path — when each matters.

### AuraOS lab
- `userspace/guest/src/bin/guest_agent.rs`, `guest_shell.rs`, `guest_init.rs`
- `userspace/guest/src/agent_ipc.rs`, `shared/src/tools.rs`, `docs/agent-core.md`

### Self-check quiz
1. Does Agent Core run in the kernel?
2. What does fail-closed mean if READY never arrives?
3. Name two example tools the shell expects.
4. How does guest IPC differ from host JSON/TCP demos?

→ [Answers — Week 15](#week-15)

---

# Block 5 — Sprints 5–8 (4 weeks)

## Week 16 — Sprint 5 (ramfb / GPU probe)

**Estimated hours:** 5–8

### Objectives
- Explain ramfb activation via fw_cfg **DMA** (not naive DATA stores).
- Separate smoke fill success from VirtIO-GPU *probe* honesty.
- Know how to run the GUI script path.

### Watch / Read
- learning-path Sprint 5; glossary ramfb, fw_cfg, DMA, Framebuffer, VirtIO-GPU, Probe/stub
- QEMU docs as needed for display backends

### Do
1. Notes: why DMA matters for fw_cfg ramfb setup.
2. Explain aloud: “probe ≠ scanout complete.”
3. Optional: run `scripts/run-qemu-gui.ps1`; match serial `ramfb smoke ok` if present.

### AuraOS lab
- `kernel/src/display.rs`, VirtIO-GPU bits in `virtio.rs`
- `userspace/shell/src/framebuffer.rs`, `scripts/run-qemu-gui.ps1`

### Self-check quiz
1. What firmware interface configures ramfb?
2. What does a smoke fill prove?
3. Is VirtIO-GPU full scanout claimed in Sprint 5?
4. Which honesty word should appear when hardware is incomplete?

→ [Answers — Week 16](#week-16)

---

## Week 17 — Sprint 6 (OTA skeleton + Pi honesty)

**Estimated hours:** 5–8

### Objectives
- Explain OTA A/B slots + channels (`os` / `agent` / `models`) at product level.
- Show why host `ota-verify` rejects unsigned manifests (fail-closed).
- Read Pi port doc as *research checklist*, not shipping drivers.

### Watch / Read
- [updates-4y.md](updates-4y.md), [hardware-port-pi5.md](hardware-port-pi5.md)
- learning-path Sprint 6; glossary OTA, A/B slots, Manifest, Signed/unsigned, Bring-up, BSP

### Do
1. Notes: integrity vs authenticity (hash vs signature) — preview for Block 6.
2. Run or read fixtures under `ota/`; confirm unsigned failure mode from docs/tool help.
3. Explain aloud: one sentence you must never claim about Pi UART today.

### AuraOS lab
- `shared/src/ota.rs`, `tools/ota-verify/`, `ota/` fixtures
- `docs/hardware-port-pi5.md`, `docs/updates-4y.md`

### Self-check quiz
1. What are A/B slots for?
2. Name the three update channels in AuraOS vocabulary.
3. What should happen to an unsigned manifest on verify?
4. Is `hardware-port-pi5.md` a driver completion report?

→ [Answers — Week 17](#week-17)

---

## Week 18 — Sprint 7 (blk, IRQ, waitpid)

**Estimated hours:** 5–8

### Objectives
- Explain VirtIO-blk discovery (e.g. `AURAAB` header) at high level.
- Contrast console IRQ drain via GIC vs poll fallback.
- Connect richer `waitpid` to fail-closed init lifecycle.

### Watch / Read
- VirtIO spec — block device skim: https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html
- learning-path Sprint 7; glossary VirtIO-blk, SPI, waitpid, Rollback

### Do
1. Notes: sector granularity; stub apply vs real slot write (S8).
2. Explain aloud: IRQ drain path for console RX.
3. Trace waitpid usage from init’s perspective (read call sites).

### AuraOS lab
- VirtIO-blk + `enable_irqs` / `handle_irq` in `kernel/src/virtio.rs`
- `kernel/src/ota.rs`, waitpid in `process.rs` / `syscall.rs`

### Self-check quiz
1. What image/header hints A/B metadata on blk?
2. What is SPI in GIC vocabulary (not “serial peripheral interface” here)?
3. Why does init need waitpid-like behavior?
4. Is on-device OTA apply complete in Sprint 7?

→ [Answers — Week 18](#week-18)

---

## Week 19 — Sprint 8 (spawn, slot write, scanout)

**Estimated hours:** 5–8

### Objectives
- Narrate kernel → init only → `SYS_SPAWN` agent/shell.
- Explain real A/B slot write + `sha256-dev` as **dev integrity**, not HSM trust.
- Describe VirtIO-GPU scanout path vs ramfb default.

### Watch / Read
- learning-path Sprint 8; glossary spawn, init/PID1, Scanout, Signed/unsigned
- Refresh agent-core boot expectations

### Do
1. Sequence: boot to shell with spawn ownership rules.
2. Explain aloud: why sha256-dev must not be marketed as production signatures.
3. Notes: who is allowed to spawn.

### AuraOS lab
- `SYS_SPAWN` in `syscall.rs` / `process.rs`; `guest_init.rs`
- blk write helpers + `init_gpu_scanout` in `virtio.rs`

### Self-check quiz
1. Who owns spawn privilege?
2. What does sha256-dev guarantee vs not guarantee?
3. What changed vs Sprint 5 for GPU?
4. Does GUI default always use VirtIO-GPU scanout?

→ [Answers — Week 19](#week-19)

---

# Block 6 — Sprints 9–11 (3 weeks)

## Week 20 — Sprint 9 (soft crypto, VB stub, TTBR0)

**Estimated hours:** 5–8

### Objectives
- Explain hash vs signature; soft ed25519 on host.
- State what the verified-boot **stub** does and does not guarantee.
- Explain TTBR0 free/destroy on process exit.

### Watch / Read
- [verified-boot.md](verified-boot.md)
- learning-path Sprint 9; glossary Verified boot, HSM, TTBR
- Optional OSTEP security/crypto skim from [OSTEP](https://pages.cs.wisc.edu/~remzi/OSTEP/) if curious — project docs win

### Do
1. Notes: SoftEd25519 vs HSM-backed keys.
2. Explain aloud: VB stub allow/deny today.
3. Why address-space destroy on exit matters.

### AuraOS lab
- `shared/src/trust.rs`, `kernel/src/vb.rs`, `kernel/src/vm.rs`
- `docs/verified-boot.md`

### Self-check quiz
1. What does a signature prove that a hash alone does not?
2. Is SoftEd25519 production HSM trust?
3. What is freed/destroyed on exit related to TTBR0?
4. Where is VB honesty documented?

→ [Answers — Week 20](#week-20)

---

## Week 21 — Sprint 10 (kernel ed25519, HSM-ready API)

**Estimated hours:** 5–8

### Objectives
- Trace kernel `no_std` soft verify into OTA/VB paths.
- Explain HSM-ready `TrustBackend` API without claiming a real HSM.
- Point at Pi prep stubs that stay honest.

### Watch / Read
- learning-path Sprint 10; Dev Plan Sprint 10 retro honesty notes
- glossary OTA, HSM, Bring-up

### Do
1. Notes: host soft verify → kernel soft verify wiring.
2. Explain aloud: why `HsmDeferred` must fail closed for production claims.
3. Find one log/string that proves stub/research, not driver.

### AuraOS lab
- `kernel/src/ota_crypto.rs`, `kernel/src/ota.rs`, `shared/src/trust.rs`
- `kernel/src/board_pi5.rs`

### Self-check quiz
1. Where does on-device soft ed25519 live?
2. What is HSM-ready vs HSM-present?
3. Should Pi stubs claim silicon UART success?
4. What is the correct posture for deferred HSM?

→ [Answers — Week 21](#week-21)

---

## Week 22 — Sprint 11 (custody, VB stages, Pi constants)

**Estimated hours:** 5–8

### Objectives
- Explain `KeyHandle` / `CustodyKind` scaffolding and soft-dev vs HSM slot handles.
- Describe VB per-stage stub checks.
- List what must be true before deleting the word *stub*.

### Watch / Read
- learning-path Sprint 11; Dev Plan Sprint 11 retro
- glossary Fail-closed, Verified boot, Probe/stub
- `docs/expected-qemu-serial.txt` Sprint 11 strings

### Do
1. Short essay (½ page): “What would have to be true before we delete *stub*?”
2. Explain aloud: custody handle vs raw key bytes.
3. Checklist: deferred items (real HSM, silicon VB, Pi drivers).

### AuraOS lab
- `KeyHandle` / `CustodyKind` in `shared/src/trust.rs`
- `kernel/src/vb.rs` stage checks; `kernel/src/board_pi5.rs` (`EarlyConsoleMap`, GIC/DT hints)
- expected serial

### Self-check quiz
1. What does a KeyHandle buy the API design?
2. Do HSM slot handles verify with live PKCS#11 today?
3. What are VB stage checks in Sprint 11?
4. Name three deferred items after Sprint 11.

→ [Answers — Week 22](#week-22)

---

# Capstone (1 week)

## Week 23 — Tiny change without AI-authored code

**Estimated hours:** 4–10

### Objectives
- Ship a **tiny**, reviewable change you personally typed (serial log line, comment-free code tweak, or small tool help text — keep it minimal and honest).
- Write a PR-style description: problem, approach, test plan, honesty notes.
- Demonstrate you can navigate build/run and serial oracle without a chatbot driving the keyboard.

### Watch / Read
- Re-read [architecture.md](architecture.md) sections you touch
- Glossary terms for any word you use in the PR text
- Git Book — [Basic Branching and Merging](https://git-scm.com/book/en/v2/Git-Branching-Basic-Branching-and-Merging)

### Do
1. **No AI writing code** for the change itself (asking “what does this function do?” is OK; pasting generated patches is not).
2. Implement something small (examples): new clearly-tagged serial log during bring-up; fix a docs typo you found while studying; add a one-line tool usage hint.
3. Run QEMU; update or cite `expected-qemu-serial.txt` if your log line is acceptance-relevant.
4. Write PR body: Summary / Test plan / Honesty (what you did *not* change).

### AuraOS lab
- Whatever files your tiny change touches — plus `scripts/run-qemu.ps1` and expected serial
- Optional: open a personal branch; do not spam production with drive-by experiments

### Self-check quiz
1. Can you explain every line you changed without reading AI commentary?
2. Did you verify on QEMU (or explain why docs-only)?
3. Did you avoid claiming stub work as production?
4. Would a mentor accept this as “you learned,” not “you prompted”?

→ [Answers — Week 23](#week-23)

---

## Study habits that work

- **Code first, blog second.** AuraOS is small enough to read.
- **Tick the checklist honestly.** A week half-done is better than a fake green box.
- **Say stub when the project says stub.** Confidence comes from honesty.
- **Prefer the allowlist.** If a Medium post disagrees with the VirtIO spec or this repo, the spec/repo wins.

---

## Answers appendix

Try each week’s quiz before opening the matching section.

### Week 0
1. The AuraOS system running under QEMU (or on hardware) — not your Windows/Linux host processes.  
2. Faster iteration, reproducible virt devices, no need for early Pi bring-up success.  
3. `docs/expected-qemu-serial.txt`.  
4. `scripts/run-qemu.ps1` (GUI variant: `run-qemu-gui.ps1`).

### Week 1
1. The caller can no longer use it; the callee owns it (unless returned).  
2. It borrows without taking ownership; works for literals and substrings.  
3. Fast typecheck without full build/run cycles.  
4. Typically kernel, userspace (guest/host), and shared (exact layout may vary — name what you saw).

### Week 2
1. No — exclusive mutable borrow rule.  
2. `Copy` duplicates bits implicitly; non-`Copy` moves.  
3. How long a reference is allowed to point at valid data.  
4. Kernels care about cycles, aliases, and accidental sharing of hardware state.

### Week 3
1. You must match `Some`/`None` (or use combinators) — no silent null.  
2. Encapsulation and navigation in large trees of modules.  
3. No allocator / bounded resources / simplicity in early bring-up.  
4. e.g. Runnable / Running / Blocked / Exited (as in AuraOS).

### Week 4
1. Unrecoverable bugs vs expected failure paths (parse errors, missing READY, verify fail).  
2. Shared behavior across types; dynamic or static dispatch of a contract.  
3. Pure functions are deterministic and easy to assert; MMIO needs integration/QEMU.  
4. If No — stay in Block 1; do not rush Foundation coding.

### Week 5
1. Lets user code run fast on hardware while the OS retains control via traps.  
2. EL0 = unprivileged guest user; EL1 = kernel.  
3. It switches privilege and enters the kernel via a well-defined entry path.  
4. e.g. `x8` = syscall number; args in `x0…`; return in `x0` (AuraOS follows AAPCS64-style).

### Week 6
1. The set of addresses a context can use — usually isolated per process.  
2. VA==PA simplifies early bring-up before full user mappings.  
3. Device registers exposed as memory addresses.  
4. Translation table base for the MMU (user/kernel tables depending on which TTBR).

### Week 7
1. Polling: early VirtIO RX; IRQ: timer preempt / later console IRQ drain.  
2. It is interrupted and may be rescheduled so others run.  
3. Races between IRQ and SVC paths corrupting shared structures.  
4. `sched: idle` (see expected serial).

### Week 8
1. The standard library (OS services, heap defaults, etc.) — you keep `core`.  
2. Hardware may require volatile access, side effects, and exact widths; compiler must not optimize like normal RAM.  
3. PL011 UART serial.  
4. e.g. Tokio host demos / host tools.

### Week 9
1. Transfers: interrupt/page-table *ideas*; does not: x86 IDT/VGA addresses, bootloaders.  
2. Exception vector table / handlers.  
3. Returns from an exception to the previous state/EL.  
4. Different architectures and machine models — addresses are platform-specific.

### Week 10
1. PL011 UART (early).  
2. Early/panic/debug path when VirtIO is down or not ready.  
3. `kernel/src/uart.rs`.  
4. `docs/expected-qemu-serial.txt`.

### Week 11
1. Saved register state across the trap so the kernel can resume the guest.  
2. EL0.  
3. `x8` (syscall number).  
4. In-kernel channels so EL0 tasks can communicate before full Agent tooling.

### Week 12
1. VirtIO-MMIO.  
2. Simpler bring-up before GIC IRQ drain.  
3. Not for the normal console path — VirtIO console is the guest console device (UART remains for early/panic).  
4. The VirtIO OASIS spec (and this repo’s driver).

### Week 13
1. ~100 Hz class tick (see timer code/docs).  
2. Forces a scheduling decision / switches away from the current EL0 context as designed.  
3. When no runnable processes remain.  
4. GIC (GICv2 on QEMU virt in AuraOS).

### Week 14
1. From the FDT `/chosen` (initrd properties) supplied by QEMU.  
2. cpio newc.  
3. ELF64.  
4. Host script `scripts/pack-initrd.ps1`.

### Week 15
1. No — userspace EL0 service.  
2. Init/system refuses to proceed as if the agent were healthy — fail closed.  
3. e.g. `help`, `system_status`.  
4. Guest uses mailbox IPC; host demos use length-prefixed JSON (often TCP) — different transports, related ideas.

### Week 16
1. fw_cfg (DMA path).  
2. Framebuffer path is alive enough to write visible pixels / serial confirmation.  
3. No — probe in S5; scanout emphasized later (S8).  
4. Probe / stub (honesty vocabulary).

### Week 17
1. Redundant slots for update/rollback style workflows.  
2. `os`, `agent`, `models`.  
3. Reject / fail closed.  
4. No — research checklist / stubs.

### Week 18
1. e.g. sector 0 `AURAAB` style header on `ab-slots.img` (see code/docs).  
2. Shared Peripheral Interrupt.  
3. Reap/wait children so lifecycle and fail-closed behavior work.  
4. No — apply stub; richer write lands more in S8.

### Week 19
1. Init (init-owned SYS_SPAWN).  
2. Dev digest integrity — not production signature/HSM authenticity.  
3. Scanout path exists beyond mere probe.  
4. Not necessarily — ramfb may still be preferred default for GUI smoke.

### Week 20
1. Authenticity / who signed — not only “bits unchanged.”  
2. No.  
3. User address space / TTBR0-associated tables (destroy/free hygiene).  
4. `docs/verified-boot.md` (+ serial honesty).

### Week 21
1. e.g. `kernel/src/ota_crypto.rs` wired through OTA/VB paths.  
2. API shaped for a future HSM backend; not a live HSM.  
3. No.  
4. Fail closed — do not pretend production keys exist.

### Week 22
1. Talk about keys without smuggling raw key bytes everywhere; future HSM slots.  
2. No — deferred/fail-closed.  
3. Boot-adjacent per-stage stub checks with clearer serial honesty.  
4. Real HSM, silicon VB signatures, real Pi UART/GIC drivers.

### Week 23
1. Yes is required — if no, undo and redo smaller.  
2. Prefer yes for code changes; docs-only must say so.  
3. Required.  
4. That is the real grading rubric.

---

_Confluence twin:_ [AuraOS Learning Course — Interactive free curriculum](https://auramislab.atlassian.net/wiki/spaces/AuraOS/pages/557199)  
_Reference track:_ [learning-path.md](learning-path.md) · [glossary.md](glossary.md)
