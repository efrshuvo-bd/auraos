# AuraOS OTA skeleton

This directory defines the **update contract** for a 4-year support window
([docs/updates-4y.md](../docs/updates-4y.md)). Sprint 6 / [SCRUM-31](https://auramislab.atlassian.net/browse/SCRUM-31)
hardens the in-repo skeleton; on-device apply is still deferred.

## Layout

| Path | Role |
|------|------|
| `channels.json` | Streams: `os`, `agent`, optional `models` |
| `slots.json` | A/B slot state + rollback flags |
| `apply_update.md` | Operator notes (inactive slot → reboot → rollback) |
| `dev-keys/` | Dev trust-anchor placeholders (**not** production secrets) |
| `fixtures/` | Sample manifests for the host verify stub |
| `../tools/ota-verify` | Host tool + tests: **reject unsigned** payloads |

## Channels

| Channel | Contents | Notes |
|---------|----------|-------|
| `os` | Kernel, drivers, shell | Flips A/B OS slot |
| `agent` | Agent Core + tool schemas | May hot-restart when ABI unchanged |
| `models` | Optional on-device packs (Tier B) | Signed; optional; may use dedicated volume |

All channels require signatures in the verify stub. Production replaces the
`dev-signed` token with real cryptography under verified boot.

## A/B + rollback (design)

1. Download signed payload for a known channel.
2. Verify with `aura-ota-verify` (or later on-device verifier) — **unsigned → reject**.
3. Write into the **inactive** slot; mark `bootable=true`.
4. Reboot; bootloader boots inactive slot.
5. Success → `successful_boot=true`, flip `active`.
6. Failure → bootloader rolls back (`rollback_on_failure` in `slots.json`).

## Host verify (Sprint 6)

```powershell
cargo test -p aura-ota-verify
cargo run -p aura-ota-verify -- ota/fixtures/unsigned-os.json   # expect reject
cargo run -p aura-ota-verify -- ota/fixtures/signed-os.json     # expect ok
```

Dev signature contract: JSON field `signature` must equal `dev-signed`.
Anything else (missing / empty / other string) is rejected.

Production devices must replace dev keys with HSM-backed keys and enforce verified boot.
