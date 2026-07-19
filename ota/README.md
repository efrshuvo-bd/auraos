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
| `fixtures/` | Sample manifests for all channels (signed + unsigned) |
| `../shared/src/ota.rs` | Channel / slot / manifest types + `verify_manifest` |
| `../tools/ota-verify` | Host CLI wrapping shared verify (**reject unsigned**) |
| `../kernel/src/ota.rs` | Boot log only: `ota: A/B not applied` |

## Channels

| Channel | Contents | Notes |
|---------|----------|-------|
| `os` | Kernel, drivers, shell | Flips A/B OS slot |
| `agent` | Agent Core + tool schemas | May hot-restart when ABI unchanged |
| `models` | Optional on-device packs (Tier B) | Signed; optional; may use dedicated volume |

All channels require signatures in the verify stub. Production replaces the
`dev-signed` token with real cryptography under verified boot (**deferred**).

## A/B + rollback (design)

1. Download signed payload for a known channel.
2. Verify with `aura-ota-verify` (or later on-device verifier) — **unsigned → reject**.
3. Write into the **inactive** slot; mark `bootable=true`.
4. Reboot; bootloader boots inactive slot.
5. Success → `successful_boot=true`, flip `active`.
6. Failure → bootloader rolls back (`rollback_on_failure` in `slots.json`).

Details: [apply_update.md](apply_update.md).

## Host verify (Sprint 6)

```powershell
.\scripts\verify-ota.ps1
# or:
cargo test -p aura-ota-verify
cargo run -p aura-ota-verify -- ota/fixtures/unsigned-os.json   # expect reject
cargo run -p aura-ota-verify -- ota/fixtures/signed-agent.json  # expect ok
```

Fixtures cover `os`, `agent`, and `models` (signed + unsigned each).

Dev signature contract: JSON field `signature` must equal `dev-signed`.
Anything else (missing / empty / other string) is rejected.

## On-device / storage (honest skeleton)

- Kernel prints `ota: A/B not applied` — no fake success.
- VirtIO-blk is **probe-only** until a real block driver exists for slot images.
- **Production crypto stays deferred** — HSM-backed keys + verified boot required
  before any device trusts an OTA write.
