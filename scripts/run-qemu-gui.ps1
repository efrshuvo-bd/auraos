# Run AuraOS under QEMU with a graphical display (ramfb visible by default)
# while keeping PL011 UART + VirtIO console muxed on stdio for serial logs.
#
# Display path (Sprint 5 / SCRUM-29):
#   -device ramfb                         → fw_cfg "etc/ramfb"; kernel maps 480x800 FB
#   -display sdl|gtk|default              → host window shows the ramfb surface
#   -VirtioGpu                            → optional VirtIO-MMIO GPU (device id 16) probe
#                                           (off by default: uninitialized virtio-gpu would
#                                           steal the window with "Guest has not initialized
#                                           the display (yet)" until queues/scanout exist)
#
# Windows / Scoop QEMU host note:
#   Scoop's GTK build often ships an empty gdk-pixbuf loaders.cache (and may warn:
#   "Could not load a pixbuf from .../Adwaita/assets/...svg"). That is a *host*
#   packaging issue — the guest ramfb path can be fine while the GTK window stays
#   stuck on QEMU's placeholder. Prefer SDL on Windows; override with -DisplayBackend.
#
# Serial path (unchanged from run-qemu.ps1):
#   -chardev stdio mux + -serial + virtconsole
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
        return @{ Args = @("-display", "sdl"); Name = "sdl"; Note = $null }
    }

    if ($Requested -eq "gtk") {
        if (-not $hasGtk) {
            throw "QEMU at $QemuPath does not list -display gtk. Available backends:`n$help"
        }
        $note = $null
        if ($env:OS -eq "Windows_NT") {
            $note = "GTK on Windows Scoop QEMU often lacks gdk-pixbuf loaders (empty loaders.cache); if the window stays on a placeholder, re-run with -DisplayBackend sdl."
        }
        return @{ Args = @("-display", "gtk"); Name = "gtk"; Note = $note }
    }

    # default: prefer SDL on Windows (avoids broken Scoop GTK / pixbuf), else gtk, else sdl, else QEMU default
    if ($env:OS -eq "Windows_NT") {
        if ($hasSdl) {
            return @{
                Args = @("-display", "sdl")
                Name = "sdl"
                Note = "Windows default: SDL (Scoop GTK often broken — Gtk-WARNING about Adwaita SVG / pixbuf loaders is a host packaging issue, not guest ramfb)."
            }
        }
        if ($hasGtk) {
            return @{
                Args = @("-display", "gtk")
                Name = "gtk"
                Note = "SDL unavailable; using GTK. If you see Gtk-WARNING about pixbuf/mime or a stuck placeholder, install a QEMU build with SDL or fix gdk-pixbuf loaders."
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

    return @{ Args = @("-display", "default"); Name = "default"; Note = "Neither sdl nor gtk listed; using -display default." }
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

Write-Host "Using QEMU: $qemu"
Write-Host "Display: $($displayArgs -join ' ') + ramfb$(if ($VirtioGpu) { ' + virtio-gpu' } else { ' (visible)' })"
if ($display.Note) {
    Write-Host $display.Note
}
Write-Host "Starting QEMU GUI (serial on this console; Ctrl+A X to exit)..."
Write-Host "Expect serial: display: ramfb mapped 480x800 ... / ramfb smoke ok ..."
if (-not $VirtioGpu) {
    Write-Host "Expect serial: display: no virtio-gpu device  (pass -VirtioGpu to probe)"
}
Write-Host "Expect window: 480x800 smoke paint (Home/Agent), not GTK placeholder text."
Write-Host "Override host UI: -DisplayBackend sdl|gtk|default"

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
    @gpuArgs `
    -kernel "$kernelBin" `
    -initrd "$initrd"
