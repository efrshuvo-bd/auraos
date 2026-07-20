# Create a tiny raw disk image for VirtIO-blk A/B slot experimentation (SCRUM-35/40).
#
# Layout:
#   Sector 0 (512 bytes):
#     [0..6)  = "AURAAB"
#     [8]     = active slot ASCII 'A' or 'B'
#     rest    = zero
#   Sector 1+: reserved for inactive-slot payload (kernel writes "INACTV" marker)
#
# Usage:
#   .\scripts\prepare-ab-disk.ps1
#   .\scripts\prepare-ab-disk.ps1 -OutPath build\ab-slots.img -ActiveSlot B

param(
    [string]$OutPath = "",
    [ValidateSet("A", "B")]
    [string]$ActiveSlot = "A",
    [int]$SizeBytes = 1MB
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
if (-not $OutPath) {
    $OutPath = Join-Path $Root "build\ab-slots.img"
}

$outDir = Split-Path -Parent $OutPath
if (-not (Test-Path -LiteralPath $outDir)) {
    New-Item -ItemType Directory -Path $outDir | Out-Null
}

if ($SizeBytes -lt 1024) {
    throw "SizeBytes must be at least 1024 (header + inactive sector)"
}

$bytes = New-Object byte[] $SizeBytes
$magic = [System.Text.Encoding]::ASCII.GetBytes("AURAAB")
[Array]::Copy($magic, 0, $bytes, 0, $magic.Length)
$bytes[8] = [byte][char]$ActiveSlot

[System.IO.File]::WriteAllBytes($OutPath, $bytes)
Write-Host "Wrote A/B disk image: $OutPath ($SizeBytes bytes, active=$ActiveSlot)"
