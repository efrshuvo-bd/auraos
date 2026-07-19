# Build AuraOS kernel for QEMU aarch64 virt.
# Guests are built first; kernel/src/guest_blobs.rs include_bytes! them (no build.rs — WDAC-safe).
$ErrorActionPreference = "Stop"
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

$Root = Split-Path -Parent $PSScriptRoot

# Prefer WDAC-safe linker on Windows when available.
if (Test-Path "$Root\scripts\fix-linker.ps1") {
    try { & "$Root\scripts\fix-linker.ps1" } catch { Write-Host "fix-linker skipped: $_" }
}

# Build EL0 guests first so include_bytes! paths exist.
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

foreach ($bin in @("guest-init", "guest-agent", "guest-shell")) {
    $p = Join-Path $GuestTarget "aarch64-unknown-none\release\$bin"
    if (-not (Test-Path -LiteralPath $p)) {
        throw "Missing guest ELF: $p"
    }
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
