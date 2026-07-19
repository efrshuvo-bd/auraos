# Host-side OTA manifest checks (Sprint 6 / SCRUM-31).
# Usage:
#   .\scripts\verify-ota.ps1
# Runs unit tests, expects unsigned fixture to fail and signed fixture to pass.

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

Write-Host "== aura-ota-verify tests =="
cargo test -p aura-ota-verify
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "== expect REJECT unsigned =="
cargo run -q -p aura-ota-verify -- ota/fixtures/unsigned-os.json
if ($LASTEXITCODE -eq 0) {
    Write-Error "unsigned fixture was accepted (should reject)"
    exit 1
}

Write-Host "== expect OK signed =="
cargo run -q -p aura-ota-verify -- ota/fixtures/signed-os.json
if ($LASTEXITCODE -ne 0) {
    Write-Error "signed fixture was rejected"
    exit $LASTEXITCODE
}

Write-Host "verify-ota.ps1: ok"
exit 0
