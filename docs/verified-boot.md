# Verified boot (AuraOS) — Sprint 9–10 stub

## Intended chain

```
bootloader (signed) → kernel image (signed) → system / inactive A/B slot (signed OTA)
```

Each stage verifies the next before handing off control or flipping `active`.
Production shipping requires **rotated ed25519 trust anchors** held in an
**HSM** (see glossary) — that custody work is **deferred**.

## What exists today (honest)

| Stage | Status |
|-------|--------|
| Host OTA verify | `dev-signed`, `sha256-dev:`, soft `ed25519:` (`shared::trust::SoftEd25519`) |
| On-device OTA verify | `sha256-dev:` + soft `ed25519:` fail-closed before VirtIO-blk slot write (Sprint 10) |
| Boot-adjacent VB stub | `kernel/src/vb.rs` — serial refuse/ok demo; not silicon |
| HSM backend | `shared::trust::HsmDeferred` — always fails closed; swap via `verify_manifest_with` |

## Stub enforcement (SCRUM-45) + on-device soft ed25519 (SCRUM-49/51)

At boot, the kernel logs:

- `vb: stub chain bootloader->kernel->system (not silicon VB / not HSM)`
- `vb: stub refuse activate (trust failed; fail-closed)` — demonstrated once
- `vb: stub trust ok (software path; HSM deferred)` — then normal OTA apply proceeds
- `ota: verify: ed25519 soft ok (on-device; not HSM / not silicon VB)` — after sha256-dev demo

If `vb::allow_activate()` is false, A/B apply must not flip the active slot.
Soft ed25519 accept is required in the Sprint 10 boot demo before apply.

## Not claimed

- Production verified boot on Pi / silicon
- HSM-backed keys
- Bootloader signature verification under QEMU virt
