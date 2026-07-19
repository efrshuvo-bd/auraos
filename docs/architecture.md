# AuraOS architecture

AuraOS is a research **agentic mobile OS**: boot → kernel → `init` → **Agent Core** (required) → shell/apps.

## Layers

1. **Boot** — QEMU `-kernel` / UEFI later; linker script at `0x40080000` for virt.
2. **Kernel (`aura-kernel`)** — UART, heap, frame allocator, **EL1 identity MMU**, VBAR/SVC/**IRQ**, GICv2 + CNTP preemption, process table (PID + Runnable/Running/Blocked/Exited), syscalls (`write`/`read`/`yield`/`exit`/`ipc_*`), in-kernel IPC, **ELF64 loader** for embedded EL0 guests, VirtIO-MMIO console TX + **polled RX**.
3. **Userspace**
   - **Guest EL0** (`userspace/guest`) — minimal `no_std` init / agent / shell ELFs embedded into the kernel image and run via `eret` + SVC.
   - **Host demos** — `aura-init` / `aura-agent` / `aura-shell` (Tokio + TCP) for richer Agent Core work on the development machine.
4. **Shared** — length-prefixed JSON IPC + tool schemas (host path).

## Agent as OS primitive

- Started immediately after IPC is ready.
- User-facing actions prefer **tool mediation** (`help`, `system_status`, `list_services`, `echo`).
- Cloud LLM optional (`AURA_LLM_*`); built-in tools work offline.
- Kernel stays small; policy + intelligence live in Agent Core.

## Host vs QEMU

| Path | What runs |
|------|-----------|
| `cargo run -p aura-shell` | Full agentic demo on host (auto-starts agent) |
| `cargo run -p aura-init` | init → agent → shell |
| `scripts/run-qemu.ps1` | Kernel serial bring-up on QEMU aarch64 virt |

## Trap / preempt return path

EL0 SVC and EL0 IRQ share one save layout and the same bridge:

`exception entry → store TrapFrame → action code → return_to_kernel → bridge_from_el0 → sched::run`

| Action | Code | Process state |
|--------|------|----------------|
| Resume | 0 | stay Running (eret) |
| Yield | 1 | → Runnable |
| Exit | 2 | → Exited (slot reusable by later `spawn`) |
| Preempt (CNTP) | 3 | → Runnable |

## Next kernel milestones

- VirtIO console IRQ → GIC (RX is still polled via `SYS_READ` / idle `virtio::poll`)
- Initrd / VirtIO-blk instead of embedding guest ELFs
- Real EL0 port of Agent Core tool loop (beyond the demo stubs)
