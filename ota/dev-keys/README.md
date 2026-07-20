# Development signing placeholders

Sprint 6 / [SCRUM-31](https://auramislab.atlassian.net/browse/SCRUM-31);
Sprint 8 / [SCRUM-41](https://auramislab.atlassian.net/browse/SCRUM-41);
Sprint 9 / [SCRUM-44](https://auramislab.atlassian.net/browse/SCRUM-44).

- **Do not** ship production secrets from this tree.
- Host verify (`tools/ota-verify` + `shared::ota::verify_manifest`) accepts:
  1. Legacy literal `dev-signed` (stub contract, **not** cryptography).
  2. Production-leaning `sha256-dev:<hex>` — SHA-256 over a canonical manifest
     payload plus a **dev** salt (`AuraOS-ota-dev-salt-v1-NOT-HSM`). Real digest
     accept/reject; **not** HSM-backed.
  3. Soft `ed25519:<hex>` — software ed25519 over the **canonical** payload
     (no salt) against the in-tree RFC8032-dev public key via
     `shared::trust::SoftEd25519` (`ed25519-compact`, no build script). **Not**
     HSM-backed. Future `HsmDeferred` backend always fails closed.
- On-device verify (`kernel/src/ota_crypto.rs`) uses the **same** salt + canonical
  form for `sha256-dev:` and gates A/B slot writes (fail-closed on unsigned / bad digest).
- Fixtures: `ota/fixtures/signed-*.json` (`dev-signed`),
  `signed-sha256-dev-os.json`, `signed-ed25519-soft-os.json`.
- **What "production" still means:** HSM-backed keys, rotated ed25519 trust
  anchors, and verified boot (bootloader → kernel → system). Names under
  `dev-keys/` and the `*-dev` / soft ed25519 prefixes are intentionally **not**
  production.
