# AuraOS architecture

AuraOS is a research **agentic mobile OS**: boot → kernel → `init` → **Agent Core** (required) → shell/apps.

Tracks `devel` after Sprint 3 (initrd guest bootstrap). Sprint 1 = VirtIO console; Sprint 2 = GICv2 + CNTP preempt.

## Layers

1. **Boot** — QEMU `-kernel build/aura-kernel.bin` (raw; Linux boot path) at `0x40080000` + `-initrd` (`-machine virt,gic-version=2`); UEFI later. ELF `-kernel` skips initrd/FDT.
2. **Kernel (`aura-kernel`)** — UART early console, heap, frame allocator, **EL1 identity MMU**, VBAR with **SVC + IRQ**, **GICv2** + **CNTP** (100 Hz) preemption, process table (PID + Runnable/Running/Blocked/Exited), syscalls (`write`/`read`/`yield`/`exit`/`ipc_*`), in-kernel IPC, **ELF64 loader**, FDT `/chosen` initrd discovery, **cpio newc** guest lookup, VirtIO-MMIO console **TX + polled RX**.
3. **Userspace**
   - **Guest EL0** (`userspace/guest`) — `guest-init` / `guest-agent` / `guest-shell` packed into `build/initrd.cpio` by `scripts/pack-initrd.ps1` (not embedded in the kernel image).
   - **Host demos** — `aura-init` / `aura-agent` / `aura-shell` (Tokio + TCP).
4. **Shared** — length-prefixed JSON IPC + tool schemas (host path).

## Bring-up sequence

1. UART → `AuraOS kernel online` (`_start` saves QEMU FDT `x0`)
2. Parse FDT `/chosen` → `linux,initrd-start` / `linux,initrd-end`
3. Heap + frame pool (`0x4400_0000`, 64 MiB)
4. Identity MMU
5. VBAR (SVC + IRQ)
6. VirtIO console probe (TX/RX queues)
7. GICv2 + CNTP arm
8. Load guests from initrd cpio → `sched::run`

Acceptance: QEMU serial reaches `sched: idle` (see `docs/expected-qemu-serial.txt`).

## I/O paths

| Path | Device | Used by |
|------|--------|---------|
| Early boot / panic / kernel `console` | PL011 UART `0x0900_0000` | EL1 only |
| Guest ELF delivery | QEMU `-initrd` (cpio newc) | Boot only |
| Guest `SYS_WRITE` | VirtIO console TX (MMIO `0x0a00_0000`+) | EL0 via syscall; UART fallback |
| Guest `SYS_READ` | VirtIO console RX (**polled**) | EL0; IRQ→GIC deferred |
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
- Cloud LLM optional (`AURA_LLM_*`); built-in tools work offline.
- Kernel stays small; policy + intelligence live in Agent Core.

## Host vs QEMU

| Path | What runs |
|------|-----------|
| `cargo run -p aura-shell` | Full agentic demo on host (auto-starts agent) |
| `cargo run -p aura-init` | init → agent → shell |
| `scripts/build-kernel.ps1` then `scripts/run-qemu.ps1` | Kernel + `-initrd build/initrd.cpio` on QEMU (`gic-version=2`, virtconsole mux) |

## Next kernel milestones

- VirtIO console IRQ → GIC (RX still polled)
- VirtIO-blk for mutable/persistent storage (initrd remains boot path)
- Real EL0 port of Agent Core tool loop (beyond the demo stubs)
