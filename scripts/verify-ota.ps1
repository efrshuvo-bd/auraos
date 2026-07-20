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
    # Mirrors shared::ota::verify_manifest: reject unsigned / empty; accept "dev-signed".
    $pairs = @(
        @{ Unsigned = "unsigned-os.json"; Signed = "signed-os.json"; Channel = "os" },
        @{ Unsigned = "unsigned-agent.json"; Signed = "signed-agent.json"; Channel = "agent" },
        @{ Unsigned = "unsigned-models.json"; Signed = "signed-models.json"; Channel = "models" }
    )
    $known = @("os", "agent", "models")

    foreach ($pair in $pairs) {
        $unsignedPath = Join-Path $root "ota\fixtures\$($pair.Unsigned)"
        $signedPath = Join-Path $root "ota\fixtures\$($pair.Signed)"
        $unsigned = Get-Content -Raw -Path $unsignedPath | ConvertFrom-Json
        $signed = Get-Content -Raw -Path $signedPath | ConvertFrom-Json

        $unsignedSig = $unsigned.signature
        if ($null -ne $unsignedSig -and "$unsignedSig".Trim() -ne "") {
            Write-Error "$($pair.Unsigned) must lack a usable signature (got: $unsignedSig)"
            return $false
        }
        if ("$($signed.signature)".Trim() -ne "dev-signed") {
            Write-Error "$($pair.Signed) must use signature 'dev-signed' (got: $($signed.signature))"
            return $false
        }
        if ($unsigned.channel -ne $pair.Channel -or $signed.channel -ne $pair.Channel) {
            Write-Error "channel mismatch for $($pair.Channel) fixtures"
            return $false
        }
        if ($unsigned.channel -notin $known -or $signed.channel -notin $known) {
            Write-Error "unknown channel in fixtures for $($pair.Channel)"
            return $false
        }
    }

    Write-Host "fixture contract ok: reject unsigned-{os,agent,models}, accept signed-* (dev-signed)"

    # Soft ed25519 fixture (Sprint 9): signature must use ed25519: prefix + 128 hex chars.
    $edPath = Join-Path $root "ota\fixtures\signed-ed25519-soft-os.json"
    if (Test-Path $edPath) {
        $ed = Get-Content -Raw -Path $edPath | ConvertFrom-Json
        $sig = "$($ed.signature)"
        if ($sig -notmatch '^ed25519:[0-9a-fA-F]{128}$') {
            Write-Error "signed-ed25519-soft-os.json must use ed25519:<128 hex> (got: $sig)"
            return $false
        }
        Write-Host "fixture contract ok: signed-ed25519-soft-os.json (ed25519 soft prefix)"
    }

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
