# Run AuraOS kernel under QEMU aarch64 virt (serial stdio)
$ErrorActionPreference = "Stop"
$env:Path = "$env:USERPROFILE\.cargo\bin;D:\scoop\shims;$env:Path"

$Root = Split-Path -Parent $PSScriptRoot
$elf = Join-Path $Root "kernel\target\aarch64-unknown-none\release\aura-kernel"

if (-not (Test-Path -LiteralPath $elf)) {
    Write-Host "Kernel not built; running build-kernel.ps1..."
    & "$PSScriptRoot\build-kernel.ps1"
}

function Find-QemuAarch64 {
    $cmd = Get-Command qemu-system-aarch64 -ErrorAction SilentlyContinue
    if ($cmd -and $cmd.Source -and (Test-Path -LiteralPath $cmd.Source)) {
        return $cmd.Source
    }

    $candidates = @(
        (Join-Path $Root "tools\qemu\qemu-system-aarch64.exe")
        (Join-Path "D:\scoop\apps\qemu\current" "qemu-system-aarch64.exe")
    )
    if ($env:SCOOP) {
        $candidates += Join-Path $env:SCOOP "apps\qemu\current\qemu-system-aarch64.exe"
    }
    $candidates += @(
        (Join-Path ${env:ProgramFiles} "qemu\qemu-system-aarch64.exe")
        (Join-Path ${env:ProgramFiles} "QEMU\qemu-system-aarch64.exe")
    )

    foreach ($c in $candidates) {
        if ($c -and (Test-Path -LiteralPath $c)) {
            return $c
        }
    }
    return $null
}

$qemu = Find-QemuAarch64
if (-not $qemu) {
    Write-Host "qemu-system-aarch64 not found."
    Write-Host "Install options:"
    Write-Host "  `$env:SCOOP='D:\scoop'; scoop install qemu"
    Write-Host "  winget install SoftwareFreedomConservancy.QEMU"
    Write-Host "  or run the Weilnetz setup and ensure qemu-system-aarch64.exe is on PATH"
    exit 1
}

Write-Host "Using QEMU: $qemu"
Write-Host "Starting QEMU (Ctrl+A X to exit qemu)..."
# VirtIO console (MMIO) for guest SYS_WRITE; UART kept for early boot.
# Mux stdio so PL011 + virtconsole share the same terminal.
# Quote the path — unquoted D:\... is parsed as a drive-scoped command.
& "$qemu" `
    -machine virt,gic-version=2 `
    -cpu cortex-a57 `
    -m 512M `
    -nographic `
    -chardev stdio,id=char0,mux=on,signal=off `
    -serial chardev:char0 `
    -mon chardev=char0 `
    -global virtio-mmio.force-legacy=false `
    -device virtio-serial-device,bus=virtio-mmio-bus.0 `
    -device virtconsole,chardev=char0 `
    -kernel "$elf"
