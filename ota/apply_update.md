# Applying an AuraOS slot update (skeleton)

Aligns with [docs/updates-4y.md](../docs/updates-4y.md) and `slots.json`.

## Happy path

1. Download signed payload for channel (`os`, `agent`, or `models`).
2. Verify signature against the device trust anchor  
   (`cargo run -p aura-ota-verify -- <manifest.json>` on host; on-device later).
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

## On-device apply stub (SCRUM-36)

Boot serial shows an explicit stub path (distinct from “nothing logged”):

- `ota: apply stub: active=A inactive=B - would switch A<->B`
- `ota: apply stub: refused unsigned (host aura-ota-verify remains authority)`
- `ota: A/B not applied (no crypto / no slot write)`

Shared helper: `shared::ota::plan_apply_stub`. Host `aura-ota-verify` remains the
authority for unsigned rejection. No inactive-slot write and no verified-boot claim.

## Production crypto (deferred)

The host stub accepts only the literal token `dev-signed`. That is **not** a
shipping trust model. Before any on-device apply:

1. Replace dev tokens with HSM-backed signatures under verified boot.
2. Keep rejecting unsigned / bad signatures before any inactive-slot write.
3. Align key rotation and EOS with [docs/updates-4y.md](../docs/updates-4y.md).
