# Development signing placeholders

Sprint 6 / [SCRUM-31](https://auramislab.atlassian.net/browse/SCRUM-31);
Sprint 8 / [SCRUM-41](https://auramislab.atlassian.net/browse/SCRUM-41).

- **Do not** ship production secrets from this tree.
- Host verify (`tools/ota-verify` + `shared::ota::verify_manifest`) accepts:
  1. Legacy literal `dev-signed` (stub contract, **not** cryptography).
  2. Production-leaning `sha256-dev:<hex>` — SHA-256 over a canonical manifest
     payload plus a **dev** salt (`AuraOS-ota-dev-salt-v1-NOT-HSM`). Real digest
     accept/reject; **not** HSM-backed and **not** ed25519.
- On-device verify (`kernel/src/ota_crypto.rs`) uses the **same** salt + canonical
  form and gates A/B slot writes (fail-closed on unsigned / bad digest).
- Fixtures: `ota/fixtures/signed-*.json` (`dev-signed`) and
  `signed-sha256-dev-os.json` (digest path).
- **What "production" still means:** HSM-backed keys, rotated ed25519 trust
  anchors, and verified boot (bootloader → kernel → system). Names under
  `dev-keys/` and the `*-dev` signature prefix are intentionally **not**
  production. Heavy ed25519 crates remain deferred where Windows WDAC blocks
  build-script host tools.
