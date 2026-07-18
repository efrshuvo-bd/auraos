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
