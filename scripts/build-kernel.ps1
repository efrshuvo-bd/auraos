# Build AuraOS kernel for QEMU aarch64 virt (embeds EL0 guest ELFs via build.rs)
$ErrorActionPreference = "Stop"
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

$Root = Split-Path -Parent $PSScriptRoot

# Prefer WDAC-safe linker on Windows when available.
if (Test-Path "$Root\scripts\fix-linker.ps1") {
    try { & "$Root\scripts\fix-linker.ps1" } catch { Write-Host "fix-linker skipped: $_" }
}

Set-Location $Root\kernel
$env:CARGO_TARGET_DIR = Join-Path $Root "kernel\target"

Write-Host "Building aura-kernel (aarch64-unknown-none)..."
cargo +nightly build -Z build-std=core,alloc --target aarch64-unknown-none --release

if ($LASTEXITCODE -ne 0) {
    throw "kernel build failed with exit $LASTEXITCODE"
}

$elf = Join-Path $Root "kernel\target\aarch64-unknown-none\release\aura-kernel"
if (-not (Test-Path $elf)) {
    throw "Kernel ELF not found at $elf"
}
Write-Host "OK: $elf"
