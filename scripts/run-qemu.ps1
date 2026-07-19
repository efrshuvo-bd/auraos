# Run AuraOS kernel under QEMU aarch64 virt (serial stdio)
$ErrorActionPreference = "Stop"
$env:Path = "$env:USERPROFILE\.cargo\bin;D:\scoop\shims;$env:Path"

$Root = Split-Path -Parent $PSScriptRoot
$kernelBin = Join-Path $Root "build\aura-kernel.bin"
$initrd = Join-Path $Root "build\initrd.cpio"

if (-not (Test-Path -LiteralPath $kernelBin) -or -not (Test-Path -LiteralPath $initrd)) {
    Write-Host "Kernel/initrd missing; running build-kernel.ps1..."
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

if (-not (Test-Path -LiteralPath $kernelBin)) {
    throw "Kernel binary not found at $kernelBin - run scripts/build-kernel.ps1"
}
if (-not (Test-Path -LiteralPath $initrd)) {
    throw "Initrd not found at $initrd - run scripts/build-kernel.ps1"
}

Write-Host "Using QEMU: $qemu"
Write-Host "Starting QEMU (Ctrl+A X to exit qemu)..."
# Raw kernel.bin (not ELF): QEMU Linux boot path loads -initrd and passes FDT in x0.
# Guests come from initrd cpio; VirtIO console for guest SYS_WRITE; UART for early boot.
# Mux stdio so PL011 + virtconsole share the same terminal.
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
    -kernel "$kernelBin" `
    -initrd "$initrd"
