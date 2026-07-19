# Run AuraOS under QEMU with a graphical display (ramfb visible by default)
# while keeping PL011 UART + VirtIO console muxed on stdio for serial logs.
#
# Success criteria (both required):
#   1) Serial: display: ramfb mapped … (fw_cfg DMA) / ramfb smoke ok …
#   2) Host window: 480x800 teal smoke paint with AURAOS / HOME / AGENT glyphs
#      — NOT "Guest has not initialized the display (yet)"
#
# Display path (Sprint 5 / SCRUM-29):
#   -device ramfb                         → fw_cfg "etc/ramfb"; kernel DMA-writes RAMFBCfg
#   -display gtk|sdl|default              → host window shows the ramfb surface
#   -VirtioGpu                            → optional VirtIO-MMIO GPU (device id 16) probe
#                                           (off by default: uninitialized virtio-gpu would
#                                           steal the window with "Guest has not initialized
#                                           the display (yet)" until queues/scanout exist)
#
# Windows / Scoop QEMU host notes:
#   - SDL often hangs during host display bring-up (black window flash / no guest serial).
#     Prefer GTK on Windows; override with -DisplayBackend sdl only if your build works.
#   - Scoop GTK may warn about empty gdk-pixbuf loaders.cache / Adwaita SVG — host packaging
#     cosmetic warnings; they do not block ramfb once the kernel uses fw_cfg DMA.
#   - Launch with WorkingDirectory + PATH = QEMU install dir so SDL/GTK DLLs resolve.
#
# Serial path (unchanged from run-qemu.ps1):
#   -chardev stdio mux + -serial + virtconsole
#   (no -nographic — that conflicts with a host display window)
#
# Headless / CI: use scripts/run-qemu.ps1 (-nographic, no GPU devices).
param(
    [switch]$VirtioGpu,
    [ValidateSet("default", "sdl", "gtk")]
    [string]$DisplayBackend = "default"
)

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

function Get-QemuDisplayHelp {
    param([Parameter(Mandatory = $true)][string]$QemuPath)
    return (& "$QemuPath" -display help 2>&1 | Out-String)
}

function Test-DisplayBackendAvailable {
    param(
        [Parameter(Mandatory = $true)][string]$HelpText,
        [Parameter(Mandatory = $true)][string]$Name
    )
    return ($HelpText -match "(?im)^\s*$([regex]::Escape($Name))\s*$")
}

function Resolve-DisplayBackend {
    param(
        [Parameter(Mandatory = $true)][string]$QemuPath,
        [Parameter(Mandatory = $true)][string]$Requested
    )

    $help = Get-QemuDisplayHelp -QemuPath $QemuPath
    $hasSdl = Test-DisplayBackendAvailable -HelpText $help -Name "sdl"
    $hasGtk = Test-DisplayBackendAvailable -HelpText $help -Name "gtk"

    if ($Requested -eq "sdl") {
        if (-not $hasSdl) {
            throw "QEMU at $QemuPath does not list -display sdl. Available backends:`n$help"
        }
        $note = $null
        if ($env:OS -eq "Windows_NT") {
            $note = "SDL on Windows Scoop QEMU often hangs before guest boot (no serial). If the window flashes/exits or serial is silent, re-run with -DisplayBackend gtk."
        }
        # gl=off avoids some Windows OpenGL/SDL bring-up stalls
        return @{ Args = @("-display", "sdl,gl=off"); Name = "sdl"; Note = $note }
    }

    if ($Requested -eq "gtk") {
        if (-not $hasGtk) {
            throw "QEMU at $QemuPath does not list -display gtk. Available backends:`n$help"
        }
        $note = $null
        if ($env:OS -eq "Windows_NT") {
            $note = "GTK on Windows Scoop QEMU may warn about gdk-pixbuf/Adwaita SVG (host packaging). Serial + ramfb should still work; ignore pixbuf warnings unless the window is blank."
        }
        return @{ Args = @("-display", "gtk"); Name = "gtk"; Note = $note }
    }

    # default: prefer GTK on Windows (Scoop SDL hangs during display bring-up — no guest serial).
    # Non-Windows: prefer gtk, then sdl, else QEMU default.
    if ($env:OS -eq "Windows_NT") {
        if ($hasGtk) {
            return @{
                Args = @("-display", "gtk")
                Name = "gtk"
                Note = 'Windows default: GTK (Scoop SDL often hangs at host display bring-up with no guest serial). Override: -DisplayBackend sdl'
            }
        }
        if ($hasSdl) {
            return @{
                Args = @("-display", "sdl,gl=off")
                Name = "sdl"
                Note = 'GTK unavailable; using SDL with gl=off. If the window flashes or serial is silent, install a QEMU build with working GTK.'
            }
        }
    } else {
        if ($hasGtk) {
            return @{ Args = @("-display", "gtk"); Name = "gtk"; Note = $null }
        }
        if ($hasSdl) {
            return @{ Args = @("-display", "sdl"); Name = "sdl"; Note = $null }
        }
    }

    return @{ Args = @("-display", "default"); Name = "default"; Note = 'Neither sdl nor gtk listed; using -display default.' }
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

$display = Resolve-DisplayBackend -QemuPath $qemu -Requested $DisplayBackend
$displayArgs = $display.Args

$gpuArgs = @()
if ($VirtioGpu) {
    $gpuArgs = @("-device", "virtio-gpu-device,bus=virtio-mmio-bus.1")
    Write-Host "VirtIO-GPU probe enabled (-VirtioGpu); window may show placeholder until queues exist."
}

$qemuDir = Split-Path -Parent $qemu
# Scoop/Weilnetz QEMU loads SDL/GTK DLLs from the install dir — keep it on PATH and as cwd.
$env:Path = "$qemuDir;$env:Path"

Write-Host "Using QEMU: $qemu"
Write-Host "QEMU dir (PATH/cwd): $qemuDir"
Write-Host "Display: $($displayArgs -join ' ') + ramfb$(if ($VirtioGpu) { ' + virtio-gpu' } else { ' (visible)' })"
if ($display.Note) {
    Write-Host $display.Note
}
Write-Host "Starting QEMU GUI (serial on this console; Ctrl+A X to exit)..."
Write-Host "Expect serial: AuraOS kernel online ..."
Write-Host "Expect serial: display: ramfb mapped 480x800 @ 0x… (fw_cfg DMA)"
Write-Host "Expect serial: display: ramfb smoke ok (solid fill + glyphs)"
if (-not $VirtioGpu) {
    Write-Host "Expect serial: display: no virtio-gpu device  (pass -VirtioGpu to probe)"
}
Write-Host "Expect window: teal 480x800 with AURAOS/HOME/AGENT text (not the QEMU placeholder)."
Write-Host "Override host UI: -DisplayBackend sdl|gtk|default"

$qemuArgs = @(
    "-machine", "virt,gic-version=2"
    "-cpu", "cortex-a57"
    "-m", "512M"
) + $displayArgs + @(
    "-device", "ramfb"
    "-chardev", "stdio,id=char0,mux=on,signal=off"
    "-serial", "chardev:char0"
    "-mon", "chardev=char0"
    "-global", "virtio-mmio.force-legacy=false"
    "-device", "virtio-serial-device,bus=virtio-mmio-bus.0"
    "-device", "virtconsole,chardev=char0"
) + $gpuArgs + @(
    "-kernel", "$kernelBin"
    "-initrd", "$initrd"
)

$exitCode = 0
Push-Location -LiteralPath $qemuDir
try {
    & "$qemu" @qemuArgs
    $exitCode = if ($null -ne $LASTEXITCODE) { [int]$LASTEXITCODE } else { 0 }
} finally {
    Pop-Location
}

Write-Host "QEMU exited with code $exitCode (LASTEXITCODE=$exitCode)"
exit $exitCode
