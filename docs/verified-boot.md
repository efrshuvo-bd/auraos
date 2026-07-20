# Verified boot (AuraOS) — Sprint 9–11 stub

## Intended chain

```
bootloader (signed) → kernel image (signed) → system / inactive A/B slot (signed OTA)
```

Each stage verifies the next before handing off control or flipping `active`.
Production shipping requires **rotated ed25519 trust anchors** held in an
**HSM** (see glossary) — that custody work is **scaffolded** (key handles) but
**not** live HSM custody.

## What exists today (honest)

| Stage | Status |
|-------|--------|
| Host OTA verify | `dev-signed`, `sha256-dev:`, soft `ed25519:` (`shared::trust::SoftEd25519`) |
| Custody scaffolding | `KeyHandle` / `CustodyKind` — soft-dev vs `HsmSlot` deferred (SCRUM-54); no fake "keys in HSM" |
| On-device OTA verify | `sha256-dev:` + soft `ed25519:` fail-closed before VirtIO-blk slot write |
| Boot-adjacent VB stub | `kernel/src/vb.rs` — per-stage stub checks + refuse/ok demo; **not** silicon |
| HSM backend | `shared::trust::HsmDeferred` — always fails closed; swap via `verify_manifest_with` |

## Stub enforcement + staged checks (SCRUM-45 / SCRUM-55)

At boot, the kernel logs:

- `vb: stub chain bootloader->kernel->system (not silicon VB / not HSM)`
- `vb: silicon path deferred (board ROM/OTP trust anchors not wired)`
- `vb: stage bootloader stub ok (not silicon VB; signature verify deferred)`
- `vb: stage kernel stub ok (not silicon VB; signature verify deferred)`
- `vb: stage system stub ok (not silicon VB; soft path only)`
- `vb: stub refuse activate (trust failed; fail-closed)` — demonstrated once
- `vb: stub trust ok (software path; HSM deferred; silicon deferred)` — then normal OTA apply
- `ota: verify: ed25519 soft ok (on-device; not HSM / not silicon VB)` — after sha256-dev demo

If `vb::allow_activate()` is false, A/B apply must not flip the active slot.
Soft ed25519 accept is required in the boot demo before apply.

## Toward silicon (still deferred)

Future board work must wire:

1. Boot ROM / OTP / fuse trust anchors (or HSM-backed anchors via custody handles)
2. Real signature verify at each `VbStage` in `kernel/src/vb.rs` (not stub-ok)
3. Fail-closed refuse that cannot be flipped by a software-only demo hook

QEMU virt continues to exercise the **stub** path only.

## Not claimed

- Production verified boot on Pi / silicon
- HSM-backed keys (handles are labels only until a real `TrustBackend`)
- Bootloader signature verification under QEMU virt
