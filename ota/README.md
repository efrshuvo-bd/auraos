# AuraOS OTA skeleton

This directory defines the **update contract** for a 4-year support window
([docs/updates-4y.md](../docs/updates-4y.md)). Sprint 8 advances on-device apply
and a production-leaning verify path beyond the Sprint 6 stub.

## Layout

| Path | Role |
|------|------|
| `channels.json` | Streams: `os`, `agent`, optional `models` |
| `slots.json` | A/B slot state + rollback flags |
| `apply_update.md` | Operator notes (inactive slot → reboot → rollback) |
| `dev-keys/` | Dev trust-anchor placeholders (**not** production secrets) |
| `fixtures/` | Sample manifests (signed / unsigned / `sha256-dev` / soft `ed25519`) |
| `../shared/src/ota.rs` | Channel / slot / manifest types + host `verify_manifest` |
| `../shared/src/trust.rs` | Soft ed25519 + HSM-deferred backend trait |
| `../tools/ota-verify` | Host CLI wrapping shared verify (**reject unsigned**) |
| `../kernel/src/ota_crypto.rs` | On-device `sha256-dev` verify (same digest algorithm) |
| `../kernel/src/ota.rs` | Fail-closed verify → VB stub gate → VirtIO-blk inactive-slot write |
| `../kernel/src/vb.rs` | Boot-adjacent verified-boot stub (refuse/ok serial) |
| `../docs/verified-boot.md` | Trust chain stages (stub vs production) |

## Channels

| Channel | Contents | Notes |
|---------|----------|-------|
| `os` | Kernel, drivers, shell | Flips A/B OS slot |
| `agent` | Agent Core + tool schemas | May hot-restart when ABI unchanged |
| `models` | Optional on-device packs (Tier B) | Signed; optional; may use dedicated volume |

## A/B + rollback (design)

1. Download signed payload for a known channel.
2. Verify with `aura-ota-verify` **and/or** on-device `ota_crypto` — **unsigned / bad digest → reject**.
3. Write into the **inactive** slot; mark `bootable=true`.
4. Reboot; bootloader boots inactive slot.
5. Success → `successful_boot=true`, flip `active`.
6. Failure → bootloader rolls back (`rollback_on_failure` in `slots.json`).

Details: [apply_update.md](apply_update.md).

## Host verify

```powershell
.\scripts\verify-ota.ps1
# or:
cargo test -p aura-ota-verify
cargo run -p aura-ota-verify -- ota/fixtures/unsigned-os.json          # reject
cargo run -p aura-ota-verify -- ota/fixtures/signed-agent.json         # ok (dev-signed)
cargo run -p aura-ota-verify -- ota/fixtures/signed-sha256-dev-os.json # ok (digest)
cargo run -p aura-ota-verify -- ota/fixtures/signed-ed25519-soft-os.json # ok (soft ed25519)
```

Accepts legacy `dev-signed`, `sha256-dev:<hex>` (dev salt), or soft `ed25519:<hex>`.
HSM / silicon verified boot remain roadmap — see `dev-keys/README.md`,
`docs/verified-boot.md`, and `docs/updates-4y.md`.

## On-device verify + storage (Sprint 8–9)

- Kernel runs the **same** `sha256-dev:` algorithm before any slot write:
  refuses unsigned, rejects a bad digest, accepts the boot-demo fixture fields.
- Serial: `ota: verify: sha256-dev ok (on-device; not HSM / not VB / not ed25519)`.
- Sprint 9 VB stub serial: `vb: stub refuse…` then `vb: stub trust ok…` before apply.
- Then writes inactive sector + flips active on VirtIO-blk when present.
- **Production still means:** HSM-backed ed25519 + verified boot chain — digests
  and soft software keys are **dev/QEMU only**.
