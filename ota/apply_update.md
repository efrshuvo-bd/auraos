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
