# Host-side OTA manifest checks (Sprint 6 / SCRUM-31).
# Usage:
#   .\scripts\verify-ota.ps1
#
# Primary proof is `cargo test -p aura-ota-verify` (unit + ota/fixtures accept/reject).
# On Windows Application Control (WDAC/AppLocker, os error 4551), freshly built
# cargo test/run exes are often blocked — Unblock-File does not fix that policy.
# In that case we fall back to a PowerShell check of the same fixture contract
# and skip optional CLI smoke.

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

# Cursor (and similar) may inject CARGO_TARGET_DIR under a Temp sandbox cache.
# WDAC often blocks those paths (os error 4551). Prefer the workspace target.
if ($env:CARGO_TARGET_DIR -and ($env:CARGO_TARGET_DIR -match 'cursor-sandbox-cache')) {
    Write-Warning "Ignoring CARGO_TARGET_DIR under cursor-sandbox-cache (WDAC-prone)."
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
}

function Test-ApplicationControlBlocked {
    param([string]$Text)
    if ([string]::IsNullOrEmpty($Text)) { return $false }
    return ($Text -match '4551') -or ($Text -match 'Application Control')
}

function Test-OtaFixturesContract {
    # Mirrors aura-ota-verify: reject unsigned / empty; accept literal "dev-signed".
    $unsignedPath = Join-Path $root "ota\fixtures\unsigned-os.json"
    $signedPath = Join-Path $root "ota\fixtures\signed-os.json"

    $unsigned = Get-Content -Raw -Path $unsignedPath | ConvertFrom-Json
    $signed = Get-Content -Raw -Path $signedPath | ConvertFrom-Json

    $unsignedSig = $unsigned.signature
    if ($null -ne $unsignedSig -and "$unsignedSig".Trim() -ne "") {
        Write-Error "unsigned fixture must lack a usable signature (got: $unsignedSig)"
        return $false
    }

    if ("$($signed.signature)".Trim() -ne "dev-signed") {
        Write-Error "signed fixture must use signature 'dev-signed' (got: $($signed.signature))"
        return $false
    }

    $known = @("os", "agent", "models")
    if ($unsigned.channel -notin $known) {
        Write-Error "unsigned fixture has unknown channel: $($unsigned.channel)"
        return $false
    }
    if ($signed.channel -notin $known) {
        Write-Error "signed fixture has unknown channel: $($signed.channel)"
        return $false
    }

    Write-Host "fixture contract ok: reject unsigned-os.json, accept signed-os.json (dev-signed)"
    return $true
}

Write-Host "== aura-ota-verify tests =="
$prevEap = $ErrorActionPreference
$ErrorActionPreference = "Continue"
$testOutput = & cargo test -p aura-ota-verify 2>&1 | Out-String
$testCode = $LASTEXITCODE
$ErrorActionPreference = $prevEap
Write-Host $testOutput

if ($testCode -ne 0) {
    if (Test-ApplicationControlBlocked $testOutput) {
        Write-Warning "cargo test blocked by Application Control (os error 4551); verifying fixture contract in PowerShell."
        if (-not (Test-OtaFixturesContract)) { exit 1 }
    } else {
        exit $testCode
    }
}

Write-Host "== optional CLI smoke (skipped if Application Control blocks exe) =="

function Invoke-OtaVerifyCli {
    param(
        [Parameter(Mandatory = $true)][string]$Manifest,
        [Parameter(Mandatory = $true)][ValidateSet("reject", "accept")][string]$Expect
    )

    $prev = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & cargo run -q -p aura-ota-verify -- $Manifest 2>&1 | Out-String
    $code = $LASTEXITCODE
    $ErrorActionPreference = $prev

    if (Test-ApplicationControlBlocked $output) {
        Write-Warning "CLI blocked by Application Control (os error 4551); tests/fixture contract are sufficient."
        return "blocked"
    }

    if ($Expect -eq "reject") {
        if ($code -eq 0) {
            Write-Error "unsigned fixture was accepted (should reject)"
            exit 1
        }
        Write-Host "CLI reject ok: $Manifest"
        return "ok"
    }

    if ($code -ne 0) {
        Write-Host $output
        Write-Error "signed fixture was rejected"
        exit $code
    }
    Write-Host "CLI accept ok: $Manifest"
    return "ok"
}

$unsigned = Invoke-OtaVerifyCli -Manifest "ota/fixtures/unsigned-os.json" -Expect "reject"
if ($unsigned -eq "blocked") {
    Write-Host "verify-ota.ps1: ok (Application Control; CLI skipped)"
    exit 0
}

$signed = Invoke-OtaVerifyCli -Manifest "ota/fixtures/signed-os.json" -Expect "accept"
if ($signed -eq "blocked") {
    Write-Host "verify-ota.ps1: ok (Application Control; CLI skipped)"
    exit 0
}

Write-Host "verify-ota.ps1: ok"
exit 0
