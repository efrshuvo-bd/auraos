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
| Host trust paths | Legacy `dev-signed` **or** `sha256-dev:<hex>` digest (dev salt; not HSM) |
| Fixtures | `ota/fixtures/{signed,unsigned}-{os,agent,models}.json` + `signed-sha256-dev-os.json` |
| Rollback story | `ota/apply_update.md` |
| Kernel on-device apply | Sprint 8: refuse unsigned, then VirtIO-blk inactive-slot write + active flip |
| VirtIO-blk for slots | QEMU read **and write** (`build/ab-slots.img`, `prepare-ab-disk.ps1`) |
| Production crypto / verified boot | **Roadmap** — replace digests/dev tokens with HSM-backed ed25519 under VB |

### QEMU A/B disk layout (SCRUM-35 / SCRUM-40)

| Item | Value |
|------|--------|
| Image | `build/ab-slots.img` via `.\scripts\prepare-ab-disk.ps1` |
| QEMU flags | `-drive file=…,if=none,format=raw,id=abdisk` + `-device virtio-blk-device,drive=abdisk,bus=virtio-mmio-bus.2` |
| Sector 0 | Magic `AURAAB`, byte 8 = active slot `'A'`/`'B'` (flipped on successful apply) |
| Sector 1 | Inactive-slot marker `INACTV` + payload stub (written on apply) |
| Slot switch | Kernel writes inactive sector + flips active when trust gate passes |

Host verify: `.\scripts\verify-ota.ps1` or `cargo test -p aura-ota-verify` —
rejects unsigned; accepts `dev-signed` and `sha256-dev:` fixtures.

### Verified-boot roadmap (honest)

1. Keep fail-closed reject of unsigned before any inactive-slot write (done).
2. Host digest path (`sha256-dev:`) proves accept/reject beyond a literal token (Sprint 8).
3. **Next:** ed25519 signatures with rotated keys (blocked on some WDAC hosts for crate build scripts — land when CI allows).
4. **Shipping:** HSM-backed keys + verified boot chain (bootloader → kernel → system) before trusting OTA on silicon.
