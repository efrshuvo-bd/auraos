# Development signing placeholders

Sprint 6 / [SCRUM-31](https://auramislab.atlassian.net/browse/SCRUM-31).

- **Do not** ship production secrets from this tree.
- Host-side verify stub (`tools/ota-verify` + `shared::ota::verify_manifest`)
  treats a payload as signed only when `signature` is the non-empty string
  `dev-signed` (dev contract, **not** cryptography).
- **Production cryptography is deferred.** Devices must use HSM-backed keys and
  real signatures (ed25519 or equivalent) under verified boot before trusting
  any OTA write — see `docs/updates-4y.md`.
