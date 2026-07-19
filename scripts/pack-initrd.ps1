# Pack guest ELFs into a cpio newc initrd for QEMU -initrd.
$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
$GuestOut = Join-Path $Root "userspace\guest\target\aarch64-unknown-none\release"
$BuildDir = Join-Path $Root "build"
$Out = Join-Path $BuildDir "initrd.cpio"

New-Item -ItemType Directory -Force -Path $BuildDir | Out-Null

$files = @(
    @{ Name = "guest-init"; Path = (Join-Path $GuestOut "guest-init") },
    @{ Name = "guest-agent"; Path = (Join-Path $GuestOut "guest-agent") },
    @{ Name = "guest-shell"; Path = (Join-Path $GuestOut "guest-shell") }
)

foreach ($f in $files) {
    if (-not (Test-Path -LiteralPath $f.Path)) {
        throw "Missing guest ELF: $($f.Path) - build guests first"
    }
}

function Write-NewcEntry {
    param(
        [System.IO.BinaryWriter]$Writer,
        [string]$Name,
        [byte[]]$Data,
        [uint32]$Mode = 33188  # 0100644 regular file
    )
    $nameBytes = [System.Text.Encoding]::ASCII.GetBytes($Name + "`0")
    $namesize = [uint32]$nameBytes.Length
    $filesize = [uint32]$Data.Length
    $hdr = "070701"
    $hdr += ("{0:x8}" -f 0)          # ino
    $hdr += ("{0:x8}" -f $Mode)      # mode
    $hdr += ("{0:x8}" -f 0)          # uid
    $hdr += ("{0:x8}" -f 0)          # gid
    $hdr += ("{0:x8}" -f 1)          # nlink
    $hdr += ("{0:x8}" -f 0)          # mtime
    $hdr += ("{0:x8}" -f $filesize)
    $hdr += ("{0:x8}" -f 0)          # devmajor
    $hdr += ("{0:x8}" -f 0)          # devminor
    $hdr += ("{0:x8}" -f 0)          # rdevmajor
    $hdr += ("{0:x8}" -f 0)          # rdevminor
    $hdr += ("{0:x8}" -f $namesize)
    $hdr += ("{0:x8}" -f 0)          # check
    $hdrBytes = [System.Text.Encoding]::ASCII.GetBytes($hdr)
    if ($hdrBytes.Length -ne 110) {
        throw "newc header length $($hdrBytes.Length) != 110"
    }
    $Writer.Write($hdrBytes)
    $Writer.Write($nameBytes)
    $pad = (4 - (($hdrBytes.Length + $nameBytes.Length) % 4)) % 4
    for ($i = 0; $i -lt $pad; $i++) { $Writer.Write([byte]0) }
    if ($Data.Length -gt 0) {
        $Writer.Write($Data)
    }
    $dpad = (4 - ($Data.Length % 4)) % 4
    for ($i = 0; $i -lt $dpad; $i++) { $Writer.Write([byte]0) }
}

$fs = [System.IO.File]::Create($Out)
$bw = New-Object System.IO.BinaryWriter $fs
try {
    foreach ($f in $files) {
        $data = [System.IO.File]::ReadAllBytes($f.Path)
        Write-NewcEntry -Writer $bw -Name $f.Name -Data $data
    }
    Write-NewcEntry -Writer $bw -Name "TRAILER!!!" -Data ([byte[]]@()) -Mode 0
} finally {
    $bw.Dispose()
    $fs.Dispose()
}

$len = (Get-Item $Out).Length
Write-Host "OK: $Out ($len bytes)"
