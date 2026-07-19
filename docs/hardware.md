# AuraOS hardware floors

AuraOS targets three profiles. Development starts on **Tier C** (QEMU). Shipping devices should meet **Tier A** (cloud agent) or **Tier B** (on-device agent).

## Tier A — Cloud-agent smooth

Minimum for responsive shell + Agent Core with models in the cloud:

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | aarch64, 8 cores @ ~2.0 GHz | 8+ cores @ 2.4 GHz+ |
| RAM | **6 GB** | **8 GB** |
| Storage | **64 GB** UFS 2.1 / fast eMMC | **128 GB** UFS 3.1 |
| Display | 1080×2400, 60 Hz | 120 Hz optional |
| GPU | GLES/Vulkan-capable | Mid-range |
| NPU | Not required | Nice-to-have |
| Network | Wi‑Fi (required for cloud agent) | Wi‑Fi + cellular |
| Battery | ~4000 mAh | 4500 mAh+ |

## Tier B — On-device agent smooth

Local 3B–7B-class quantized models + tool loops:

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | aarch64, 8 cores @ ~2.4 GHz | Flagship mid/high |
| RAM | **12 GB** | **16 GB** |
| Storage | **128 GB** UFS | **256 GB** |
| NPU | **≥ ~20 TOPS** INT8 (or GPU equiv.) | 40+ TOPS |
| GPU | Mid/high | Strong Adreno/Mali |
| Thermal | Sustained inference usable | Vapor chamber / good chassis |
| Network | Wi‑Fi (hybrid fallback) | Wi‑Fi + cellular |

## Tier C — Research / bring-up

| Platform | Spec |
|----------|------|
| QEMU `virt` aarch64 | 4–8 GB guest RAM |
| Raspberry Pi 4/5 | 8 GB for bring-up; cloud agent OK |

Pi 5 bring-up checklist, QEMU gaps, and feature flags: [hardware-port-pi5.md](hardware-port-pi5.md) (Sprint 6 / SCRUM-30). Default runtime remains QEMU virt — the Pi port is research stubs only.

## Product stance

Prefer SoCs with obtainable boot docs, long BSP life, and an NPU so AuraOS can promise Tier B + a **4-year** update window (see [updates-4y.md](updates-4y.md)).
