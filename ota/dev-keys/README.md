# Development signing placeholders

Sprint 6 / [SCRUM-31](https://auramislab.atlassian.net/browse/SCRUM-31).

- **Do not** ship production secrets from this tree.
- Host-side verify stub (`tools/ota-verify`) treats a payload as signed only when
  `signature` is the non-empty string `dev-signed` (dev contract, not crypto).
- Production devices must use HSM-backed keys and real signatures (ed25519 or
  equivalent) under verified boot — see `docs/updates-4y.md`.
