# Run AuraOS kernel under QEMU aarch64 virt (serial stdio)
$ErrorActionPreference = "Stop"
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

$Root = Split-Path -Parent $PSScriptRoot
$elf = Join-Path $Root "kernel\target\aarch64-unknown-none\release\aura-kernel"

if (-not (Test-Path $elf)) {
    Write-Host "Kernel not built; running build-kernel.ps1..."
    & "$PSScriptRoot\build-kernel.ps1"
}

function Find-QemuAarch64 {
    $cmd = Get-Command qemu-system-aarch64 -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }

    $candidates = @(
        (Join-Path $Root "tools\qemu\qemu-system-aarch64.exe"),
        "D:\scoop\apps\qemu\current\qemu-system-aarch64.exe",
        "$env:SCOOP\apps\qemu\current\qemu-system-aarch64.exe",
        "C:\Program Files\qemu\qemu-system-aarch64.exe",
        "C:\Program Files\QEMU\qemu-system-aarch64.exe"
    ) | Where-Object { $_ -and (Test-Path $_) }

    if ($candidates) { return $candidates[0] }
    return $null
}

$qemu = Find-QemuAarch64
if (-not $qemu) {
    Write-Host "qemu-system-aarch64 not found."
    Write-Host "Install options:"
    Write-Host "  scoop install qemu   (recommended on low-C: disks: set SCOOP=D:\scoop)"
    Write-Host "  winget install SoftwareFreedomConservancy.QEMU"
    Write-Host "  or run the Weilnetz setup and ensure qemu-system-aarch64.exe is on PATH"
    exit 1
}

Write-Host "Using QEMU: $qemu"
Write-Host "Starting QEMU (Ctrl+A X to exit qemu)..."
& $qemu `
  -machine virt `
  -cpu cortex-a57 `
  -m 512M `
  -nographic `
  -serial mon:stdio `
  -kernel $elf
