# AuraOS 4-year update commitment

A **4-year support window** is a product requirement. It drives partition layout, signing, and Agent Core versioning.

## What 4 years covers

- Security patches: kernel, boot chain, Agent Core, system services (monthly target)
- Agent Core + model-runtime updates without full OS reflash when possible
- IPC / tool ABI compatibility within a major OS line
- OTA delivery until end-of-support (**EOS = ship date + 4 years** per device generation)

## Technical requirements

1. **A/B slots** — update inactive slot, reboot, rollback on failed boot  
2. **Verified boot** — signed bootloader → kernel → system; reject unsigned OTAs  
3. **Channels**
   - `os` — kernel, drivers, shell  
   - `agent` — Agent Core + tool schemas  
   - `models` — optional on-device weights  
4. **Storage reserve** — ≥15% free on system volume; dedicated `models` volume on Tier B  
5. **Hardware generation contract** — SoC, min RAM/storage, EOS date  
6. **Driver policy** — prefer mainline / in-tree; avoid dead-end blobs  
7. **Optional update health** — boot success signals (privacy-documented)

## Cadence

| Stream | Cadence |
|--------|---------|
| Security | Monthly |
| Agent tools/policy | Biweekly → monthly once stable |
| OS feature | Quarterly |
| Model packs (Tier B) | As needed, signed, optional |

## Skeleton in-tree

See [`ota/`](../ota/) for channel manifests, A/B slot metadata, and dev signing placeholders.

| Piece | Status |
|-------|--------|
| Channels `os` / `agent` / `models` | In `ota/channels.json` + `shared::ota::Channel` |
| A/B slot metadata | `ota/slots.json` + `shared::ota::{SlotId, AbSlots}` |
| Host reject-unsigned | `aura-ota-verify` (`shared::ota::verify_manifest`) |
| Host trust paths | Legacy `dev-signed`, `sha256-dev:<hex>` digest (dev salt), or soft `ed25519:<hex>` (`shared::trust`; not HSM) |
| Fixtures | `ota/fixtures/{signed,unsigned}-{os,agent,models}.json` + `signed-sha256-dev-os.json` + `signed-ed25519-soft-os.json` |
| Rollback story | `ota/apply_update.md` |
| Kernel on-device apply | Sprint 8: on-device `sha256-dev` verify (fail-closed), then VirtIO-blk inactive-slot write + active flip; Sprint 9 VB stub gate |
| VirtIO-blk for slots | QEMU read **and write** (`build/ab-slots.img`, `prepare-ab-disk.ps1`) |
| Production crypto / verified boot | **Partial** — soft ed25519 host + VB stub landed; **shipping** still needs HSM-backed keys under real VB |

### QEMU A/B disk layout (SCRUM-35 / SCRUM-40)

| Item | Value |
|------|--------|
| Image | `build/ab-slots.img` via `.\scripts\prepare-ab-disk.ps1` |
| QEMU flags | `-drive file=…,if=none,format=raw,id=abdisk` + `-device virtio-blk-device,drive=abdisk,bus=virtio-mmio-bus.2` |
| Sector 0 | Magic `AURAAB`, byte 8 = active slot `'A'`/`'B'` (flipped on successful apply) |
| Sector 1 | Inactive-slot marker `INACTV` + payload stub (written on apply) |
| Slot switch | Kernel writes inactive sector + flips active **only after** on-device verify succeeds |

Host verify: `.\scripts\verify-ota.ps1` or `cargo test -p aura-ota-verify` —
rejects unsigned; accepts `dev-signed`, `sha256-dev:`, and soft `ed25519:` fixtures.

### On-device verify (SCRUM-41) + soft ed25519 / VB (Sprint 9)

1. Fail-closed reject of unsigned **and** bad digests before any inactive-slot write (done; serial proof).
2. Host + on-device `sha256-dev:` digest path (same salt/canonical form in `shared::ota` and `kernel/src/ota_crypto.rs`) (done).
3. Host soft `ed25519:` via `shared::trust::SoftEd25519` + `ed25519-compact` (no build script); `HsmDeferred` backend shape present but fails closed (SCRUM-44).
4. Boot-adjacent VB stub (`kernel/src/vb.rs`, `docs/verified-boot.md`) demonstrates refuse-then-ok before apply (SCRUM-45).
5. **What "production" still means:** rotated **ed25519** trust anchors in an **HSM**, and a real verified boot chain on silicon. Soft software keys are **dev/QEMU only**.
6. **Shipping:** HSM + silicon VB before trusting OTA on device.
