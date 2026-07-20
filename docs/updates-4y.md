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
| Host reject-unsigned | `aura-ota-verify` (uses `shared::ota::verify_manifest`) |
| Fixtures | `ota/fixtures/{signed,unsigned}-{os,agent,models}.json` |
| Rollback story | `ota/apply_update.md` |
| Kernel on-device apply | **Stub** — logs would-switch A↔B + refuses unsigned; still `A/B not applied` |
| VirtIO-blk for slots | QEMU read path for sector 0 (`build/ab-slots.img`, `prepare-ab-disk.ps1`) |
| Production crypto / verified boot | **Deferred** — replace `dev-signed` with HSM-backed signatures |

### QEMU A/B disk layout (SCRUM-35)

| Item | Value |
|------|--------|
| Image | `build/ab-slots.img` via `.\scripts\prepare-ab-disk.ps1` |
| QEMU flags | `-drive file=…,if=none,format=raw,id=abdisk` + `-device virtio-blk-device,drive=abdisk,bus=virtio-mmio-bus.2` |
| Sector 0 | Magic `AURAAB`, byte 8 = active slot `'A'`/`'B'` |
| Slot switch | Kernel OTA apply stub only (no write / no reboot flip yet) |

Host verify (Sprint 6): `.\scripts\verify-ota.ps1` or `cargo test -p aura-ota-verify` —
rejects unsigned payloads per the `dev-signed` contract in `ota/dev-keys/README.md`.

**Production cryptography stays deferred:** the stub must never be mistaken for
device trust. Shipping devices need verified boot + real signatures (ed25519 or
equivalent) before any OTA write to an inactive slot.
