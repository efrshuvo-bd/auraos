# Build AuraOS kernel for QEMU aarch64 virt.
# Guests are packed into build/initrd.cpio (not embedded in the kernel image).
$ErrorActionPreference = "Stop"
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

$Root = Split-Path -Parent $PSScriptRoot

# Prefer WDAC-safe linker on Windows when available.
if (Test-Path "$Root\scripts\fix-linker.ps1") {
    try { & "$Root\scripts\fix-linker.ps1" } catch { Write-Host "fix-linker skipped: $_" }
}

# Build EL0 guests, then pack initrd.
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

Write-Host "Packing initrd (cpio newc)..."
& "$PSScriptRoot\pack-initrd.ps1"

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

# QEMU only loads -initrd / passes FDT in x0 for "Linux" images, not ELF.
# Emit a raw binary so arm_load_kernel takes the aarch64 Image path (is_linux=1).
$BuildDir = Join-Path $Root "build"
New-Item -ItemType Directory -Force -Path $BuildDir | Out-Null
$bin = Join-Path $BuildDir "aura-kernel.bin"
$objcopy = $null
foreach ($c in @(
        (Join-Path ${env:ProgramFiles} "Microsoft Visual Studio\18\Community\VC\Tools\Llvm\bin\llvm-objcopy.exe"),
        (Join-Path ${env:ProgramFiles} "Microsoft Visual Studio\17\Community\VC\Tools\Llvm\bin\llvm-objcopy.exe"),
        (Join-Path $env:USERPROFILE ".cargo\bin\rust-objcopy.exe")
    )) {
    if ($c -and (Test-Path -LiteralPath $c)) { $objcopy = $c; break }
}
if (-not $objcopy) {
    $cmd = Get-Command llvm-objcopy -ErrorAction SilentlyContinue
    if ($cmd) { $objcopy = $cmd.Source }
}
if (-not $objcopy) {
    throw "llvm-objcopy/rust-objcopy not found (needed for aura-kernel.bin)"
}
& $objcopy -O binary $elf $bin
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $bin)) {
    throw "objcopy failed producing $bin"
}

$initrd = Join-Path $BuildDir "initrd.cpio"
Write-Host "OK: $elf"
Write-Host "OK: $bin"
Write-Host "OK: $initrd"
