# Agent Core

Privileged AuraOS system service (`agent.core`).

## Responsibilities

- Session memory (persisted under `.aura/agent-memory.json` on host)
- Natural-language interface to system tools
- Policy gate for which tools may run
- Pluggable model backend (mock / OpenAI-compatible cloud / future on-device)

## Built-in tools

Single source of names (host + guest). Guest IDs live in `userspace/guest/src/agent_ipc.rs` and are mirrored as `shared::tools::TOOL_ID_*`.

| Tool | Guest id | Description |
|------|----------|-------------|
| `help` | 1 | List tools |
| `system_status` | 2 | OS/agent status |
| `list_services` | 3 | Running services |
| `echo` | 4 | Echo text |

## IPC

| Track | Transport | Framing |
|-------|-----------|---------|
| Host | TCP `127.0.0.1:7420` | Length-prefixed JSON (`shared::ipc`) |
| Guest EL0 | In-kernel u64 mailboxes | Opcodes on ch2 READY / ch3 REQ / ch4 RESP |

Guest protocol (Sprint 4):

1. Agent posts `READY` (`0xA11E`) on channel 2 and enters a yield loop.
2. Shell waits for READY (fail-closed on timeout).
3. Shell posts tool id on channel 3; agent replies `OK|tool_id` on channel 4.
4. Shell may post `SHUTDOWN` (`0xDEAD`) so agent exits and QEMU can reach `sched: idle`.

## Failure policy

`init` treats Agent Core as **required**. On guest EL0:

- `guest-init` waits for READY; if missing, prints `FAIL CLOSED` and exits without treating the session as healthy.
- `guest-shell` also refuses a normal session without READY / successful `help` + `system_status`.

Sprint 7 adds a thin non-blocking `SYS_WAITPID` (guest `waitpid_noblock`). Kernel
still loads init/agent/shell from initrd; init exercises waitpid and keeps
READY-based fail-closed. Full init-owned spawn of agent/shell remains deferred.
