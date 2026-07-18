# Applying an AuraOS slot update (skeleton)

1. Download signed payload for channel (`os`, `agent`, or `models`).
2. Verify signature against the device trust anchor.
3. Write payload into the **inactive** A/B slot.
4. Mark inactive slot `bootable=true`, reboot.
5. On success, set `successful_boot=true` and flip `active`.
6. On failure, bootloader rolls back to previous slot.

Agent-only updates may hot-restart `agent.core` without flipping OS slots when the tool ABI is unchanged.
