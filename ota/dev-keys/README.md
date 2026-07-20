# Development signing placeholders

Sprint 6 / [SCRUM-31](https://auramislab.atlassian.net/browse/SCRUM-31);
Sprint 8 / [SCRUM-41](https://auramislab.atlassian.net/browse/SCRUM-41).

- **Do not** ship production secrets from this tree.
- Host verify (`tools/ota-verify` + `shared::ota::verify_manifest`) accepts:
  1. Legacy literal `dev-signed` (stub contract, **not** cryptography).
  2. Production-leaning `sha256-dev:<hex>` — SHA-256 over a canonical manifest
     payload plus a **dev** salt (`AuraOS-ota-dev-salt-v1-NOT-HSM`). Real digest
     accept/reject; **not** HSM-backed and **not** ed25519 yet.
- Fixtures: `ota/fixtures/signed-*.json` (`dev-signed`) and
  `signed-sha256-dev-os.json` (digest path).
- **Verified boot / ed25519 / HSM** remain the shipping roadmap — see
  `docs/updates-4y.md`. Heavy ed25519 crates are deferred where Windows WDAC
  blocks build-script host tools; the digests API keeps fail-closed verify
  honest until that lands.
