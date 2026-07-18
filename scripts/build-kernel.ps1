# Build AuraOS kernel for QEMU aarch64 virt (embeds EL0 guest ELFs via build.rs)
$ErrorActionPreference = "Stop"
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

$Root = Split-Path -Parent $PSScriptRoot

# Prefer WDAC-safe linker on Windows when available.
if (Test-Path "$Root\scripts\fix-linker.ps1") {
    try { & "$Root\scripts\fix-linker.ps1" } catch { Write-Host "fix-linker skipped: $_" }
}

# Build EL0 guests first (kernel build.rs only embeds; avoids nested cargo under WDAC).
$GuestDir = Join-Path $Root "userspace\guest"
$GuestTarget = Join-Path $GuestDir "target"
Write-Host "Building aura-guest EL0 binaries..."
Push-Location $GuestDir
try {
    $env:CARGO_TARGET_DIR = $GuestTarget
    Remove-Item Env:CARGO_ENCODED_RUSTFLAGS -ErrorAction SilentlyContinue
    Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue
    Remove-Item Env:CARGO_TARGET_AARCH64_UNKNOWN_NONE_RUSTFLAGS -ErrorAction SilentlyContinue
    rustup run nightly cargo build -Z build-std=core --release --target aarch64-unknown-none --bins
    if ($LASTEXITCODE -ne 0) {
        throw "aura-guest build failed with exit $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

Set-Location $Root\kernel
$env:CARGO_TARGET_DIR = Join-Path $Root "kernel\target"

# WDAC may block freshly linked build-script EXEs until unblocked.
Get-ChildItem (Join-Path $env:CARGO_TARGET_DIR "release\build") -Recurse -Filter "build-script-build*" -ErrorAction SilentlyContinue |
    ForEach-Object { Unblock-File -LiteralPath $_.FullName -ErrorAction SilentlyContinue }

Write-Host "Building aura-kernel (aarch64-unknown-none)..."
cargo +nightly build -Z build-std=core,alloc --target aarch64-unknown-none --release

if ($LASTEXITCODE -ne 0) {
    # Retry once after unblocking build scripts (common on WDAC-locked Windows).
    Get-ChildItem (Join-Path $env:CARGO_TARGET_DIR "release\build") -Recurse -Filter "build-script-build*" -ErrorAction SilentlyContinue |
        ForEach-Object { Unblock-File -LiteralPath $_.FullName -ErrorAction SilentlyContinue }
    cargo +nightly build -Z build-std=core,alloc --target aarch64-unknown-none --release
    if ($LASTEXITCODE -ne 0) {
        throw "kernel build failed with exit $LASTEXITCODE"
    }
}

$elf = Join-Path $Root "kernel\target\aarch64-unknown-none\release\aura-kernel"
if (-not (Test-Path $elf)) {
    throw "Kernel ELF not found at $elf"
}
Write-Host "OK: $elf"
