# Run AuraOS under QEMU with a graphical display (ramfb + VirtIO-GPU probe)
# while keeping PL011 UART + VirtIO console muxed on stdio for serial logs.
#
# Display path (Sprint 5 / SCRUM-29):
#   -device ramfb                         → fw_cfg "etc/ramfb"; kernel maps 480x800 FB
#   -device virtio-gpu-device,...          → VirtIO-MMIO GPU (device id 16) probe
#   -display gtk (or sdl / default)       → host window; falls back if gtk missing
#
# Serial path (unchanged from run-qemu.ps1):
#   -chardev stdio mux + -serial + virtconsole
#
# Headless / CI: use scripts/run-qemu.ps1 (-nographic, no GPU devices).
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
    exit 1
}

if (-not (Test-Path -LiteralPath $kernelBin)) {
    throw "Kernel binary not found at $kernelBin - run scripts/build-kernel.ps1"
}
if (-not (Test-Path -LiteralPath $initrd)) {
    throw "Initrd not found at $initrd - run scripts/build-kernel.ps1"
}

# Prefer gtk; many Windows builds ship sdl or a default display backend.
$displayArgs = @("-display", "gtk")
$help = & "$qemu" -display help 2>&1 | Out-String
if ($help -notmatch "(?i)\bgtk\b") {
    if ($help -match "(?i)\bsdl\b") {
        $displayArgs = @("-display", "sdl")
    } else {
        $displayArgs = @("-display", "default")
    }
}

Write-Host "Using QEMU: $qemu"
Write-Host "Display: $($displayArgs -join ' ')"
Write-Host "Starting QEMU GUI (serial on this console; Ctrl+A X to exit)..."
Write-Host "Expect serial: display: virtio-gpu ... and/or display: ramfb mapped 480x800 ..."

& "$qemu" `
    -machine virt,gic-version=2 `
    -cpu cortex-a57 `
    -m 512M `
    @displayArgs `
    -device ramfb `
    -chardev stdio,id=char0,mux=on,signal=off `
    -serial chardev:char0 `
    -mon chardev=char0 `
    -global virtio-mmio.force-legacy=false `
    -device virtio-serial-device,bus=virtio-mmio-bus.0 `
    -device virtconsole,chardev=char0 `
    -device virtio-gpu-device,bus=virtio-mmio-bus.1 `
    -kernel "$kernelBin" `
    -initrd "$initrd"
