# AuraOS

Project root: `D:\WorkSpace\SysemDev\auraos`

From-zero **agentic AI mobile OS** research project. AuraOS owns its boot path, kernel, IPC, and userspace — with **Agent Core** as a first-class system citizen (started by `init`, privileged, policy-gated tools).

## Quick start (host demo)

Requires Rust nightly (see `rust-toolchain.toml`).

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
cargo run -p aura-shell
```

This starts **init → Agent Core → shell** on the host (Phase 3–4 demo path). Ask the agent things like `help`, `status`, or `echo hello`.

Optional cloud LLM (OpenAI-compatible):

```powershell
$env:AURA_LLM_API_KEY = "sk-..."
$env:AURA_LLM_BASE_URL = "https://api.openai.com/v1"
$env:AURA_LLM_MODEL = "gpt-4o-mini"
cargo run -p aura-shell
```

## Kernel on QEMU (aarch64)

```powershell
.\scripts\build-kernel.ps1
.\scripts\run-qemu.ps1
```

Requires [QEMU](https://www.qemu.org/) with `qemu-system-aarch64` on `PATH`.

You should see: `AuraOS kernel online` on the serial console (see [docs/expected-qemu-serial.txt](docs/expected-qemu-serial.txt)).

**Windows note:** WDAC often blocks rustup’s `rust-lld` (os error 4551). Prefer Visual Studio’s `ld.lld` via `.\scripts\fix-linker.ps1` (updates both `.cargo/config.toml` and `kernel/.cargo/config.toml`). Copied `tools\lld.exe` is frequently blocked too.

**QEMU note:** If `winget`’s QEMU package is tools-only (no `qemu-system-*`), install a full build with Scoop on D: when C: is tight: `$env:SCOOP='D:\scoop'; scoop install qemu`.

## Layout

```
auraos/
  kernel/           # no_std aarch64 kernel (QEMU virt)
  agent/            # Agent Core service
  userspace/init/   # PID 1
  userspace/shell/  # Framebuffer-style + serial agent UI
  shared/           # IPC protocol + tool schemas
  boot/             # Linker scripts / boot stubs
  docs/             # Architecture, hardware tiers, 4y updates
  ota/              # A/B OTA skeleton + update channels
  scripts/          # build / QEMU helpers
```

## Hardware & updates

- Hardware floors: [docs/hardware.md](docs/hardware.md) (Tier A cloud / Tier B on-device / Tier C bring-up)
- 4-year support: [docs/updates-4y.md](docs/updates-4y.md)
- Board port (Pi 5): [docs/hardware-port-pi5.md](docs/hardware-port-pi5.md)

## License

MIT — see [LICENSE](LICENSE).
