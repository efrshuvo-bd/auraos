# AuraOS architecture

AuraOS is a research **agentic mobile OS**: boot → kernel → `init` → **Agent Core** (required) → shell/apps.

## Layers

1. **Boot** — QEMU `-kernel` / UEFI later; linker script at `0x40080000` for virt.
2. **Kernel (`aura-kernel`)** — UART, heap, frame allocator, **EL1 identity MMU**, VBAR/SVC, cooperative scheduler, syscalls (`write`/`read`/`yield`/`exit`/`ipc_*`), in-kernel IPC mailboxes, **ELF64 loader** for embedded EL0 guests, VirtIO-MMIO console TX + **polled RX** (IRQ→GIC deferred).
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

## Next kernel milestones

- VirtIO console IRQ → GIC (RX is polled today via `SYS_READ` / idle `virtio::poll`)
- Preemptive timer IRQ + richer process table
- Initrd / VirtIO-blk instead of embedding guest ELFs
- Real EL0 port of Agent Core tool loop (beyond the demo stubs)
