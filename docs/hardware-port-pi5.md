# Hardware port: Raspberry Pi 5

Phase 5 bring-up target for AuraOS (Tier C → Tier A cloud agent).

## Why Pi 5

- Documented boot flow (EEPROM → firmware → kernel)
- aarch64, enough RAM for Agent Core host-style userspace during port
- Community device trees / UART

## Port checklist

1. **UART** — early console on the Pi 5 debug UART (document GPIO pins in board notes)
2. **Device tree** — parse memory map; feed `frame::init`
3. **Framebuffer** — HDMI framebuffer or VC4 path stub
4. **Timer / GIC** — replace soft ticks with architected timer + IRQ
5. **Storage** — SD/eMMC for A/B slot images under `ota/`
6. **Network** — Ethernet/Wi‑Fi later for cloud Agent Core + OTA

## Minimum for “AuraOS on Pi 5” milestone

- Boots to serial: `AuraOS kernel online`
- Scheduler runs `init` / `agent.core` / `shell` tasks (or host-nfs userspace)
- OTA metadata reads A/B slot from `ota/slots.json`

## Not in v0

- Full GPU compositor, camera, or vendor NPU acceleration (Tier B needs a phone SoC class device)
