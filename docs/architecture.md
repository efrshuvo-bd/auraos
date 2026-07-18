# AuraOS architecture

AuraOS is a research **agentic mobile OS**: boot → kernel → `init` → **Agent Core** (required) → shell/apps.

## Layers

1. **Boot** — QEMU `-kernel` / UEFI later; linker script at `0x40080000` for virt.
2. **Kernel (`aura-kernel`)** — UART, heap, frame allocator, VM stub, timer, cooperative scheduler, syscalls (`write`/`yield`/`exit`/`ipc_*`), in-kernel IPC mailboxes, simulated userspace tasks.
3. **Userspace**
   - `aura-init` — PID 1; fails closed if Agent Core dies at start.
   - `aura-agent` — Agent Core: tools, memory, pluggable LLM backend.
   - `aura-shell` — home UI + always-on agent overlay (serial + PPM framebuffer demo).
4. **Shared** — length-prefixed JSON IPC + tool schemas.

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

- Real EL1 page tables + userspace EL0
- VirtIO console / GPU / input
- ELF loader for init/agent/shell inside the guest
