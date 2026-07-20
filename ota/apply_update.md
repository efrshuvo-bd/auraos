# Applying an AuraOS slot update

Aligns with [docs/updates-4y.md](../docs/updates-4y.md) and `slots.json`.

## Happy path

1. Download signed payload for channel (`os`, `agent`, or `models`).
2. Verify signature against the device trust anchor  
   (`cargo run -p aura-ota-verify -- <manifest.json>` on host; on-device gate in kernel).
3. **Reject** if unsigned or signature invalid — do not write any slot.
4. Write payload into the **inactive** A/B slot.
5. Mark inactive slot `bootable=true`, reboot.
6. On success, set `successful_boot=true` and flip `active`.

## Rollback story

- `slots.json` sets `rollback_on_failure: true`.
- If the newly booted slot fails health checks (no `successful_boot` within the
  bootloader’s try count), the bootloader marks that slot `bootable=false` and
  reboots the previous `active` slot.
- Agent-only updates may hot-restart `agent.core` without flipping OS slots when
  the tool ABI is unchanged; failed agent restart should leave the last known
  good agent binary in place (no half-written active slot).

## Channels

See `channels.json` and `README.md`. `models` is optional and must still be signed.
Typed in code as `shared::ota::Channel` (`os` | `agent` | `models`).

## On-device apply (Sprint 8 / SCRUM-40)

Boot serial distinguishes refuse vs real write:

- `ota: verify: refused unsigned (fail-closed before slot write)`
- `ota: verify: boot-demo trust ok (dev key; not HSM / not VB)`
- `ota: apply real: wrote inactive=B flipped active=B (virtio-blk)` (letters depend on prior active)
- `ota: A/B slot write ok (unsigned still refused above)`

Disk: sector 0 `AURAAB` + active byte; sector 1 `INACTV` marker. Shared planner:
`shared::ota::plan_apply_stub`. Host `aura-ota-verify` remains the fixture authority.

## Trust / crypto (SCRUM-41)

Host accepts:

- Legacy `dev-signed` token
- `sha256-dev:<hex>` digest over canonical payload + **dev** salt (real accept/reject)

**Not yet:** HSM-backed ed25519 or full verified boot. Roadmap in `docs/updates-4y.md`
and `ota/dev-keys/README.md`.
