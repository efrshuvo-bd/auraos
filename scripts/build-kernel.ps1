# Build AuraOS kernel for QEMU aarch64 virt
$ErrorActionPreference = "Stop"
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root\kernel

Write-Host "Building aura-kernel (aarch64-unknown-none)..."
cargo +nightly build -Z build-std=core,alloc --target aarch64-unknown-none --release

$elf = Join-Path $Root "kernel\target\aarch64-unknown-none\release\aura-kernel"
if (-not (Test-Path $elf)) {
    # Windows may not add .exe for none target
    throw "Kernel ELF not found at $elf"
}
Write-Host "OK: $elf"
