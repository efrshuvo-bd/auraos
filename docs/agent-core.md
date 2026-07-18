# Agent Core

Privileged AuraOS system service (`agent.core`).

## Responsibilities

- Session memory (persisted under `.aura/agent-memory.json` on host)
- Natural-language interface to system tools
- Policy gate for which tools may run
- Pluggable model backend (mock / OpenAI-compatible cloud / future on-device)

## Built-in tools

| Tool | Description |
|------|-------------|
| `help` | List tools |
| `system_status` | OS/agent status |
| `list_services` | Running services |
| `echo` | Echo text |

## IPC

TCP length-prefixed JSON on host (`127.0.0.1:7420` by default). Same schema lives in `shared` for future in-guest virtio/IPC ports.

## Failure policy

`init` treats Agent Core as **required**. If it exits during startup, init fails closed and does not present a normal shell session.
