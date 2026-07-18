# Prefer a WDAC-allowed linker for aarch64-unknown-none (VS LLVM ld.lld).
# Copied rust-lld / tools/lld.exe is often blocked (os error 4551).
$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
$configDir = Join-Path $Root "kernel\.cargo"
New-Item -ItemType Directory -Force -Path $configDir | Out-Null

$ldScript = (Join-Path $Root "kernel\aarch64-qemu.ld") -replace '\\', '\\'

$vsLld = "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\Llvm\bin\ld.lld.exe"
$mingwRoots = Get-ChildItem "$env:LOCALAPPDATA\Microsoft\WinGet\Packages" -Directory -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -like "MartinStorsjo.LLVM-MinGW*" }

$linker = $null
if (Test-Path $vsLld) {
    $linker = $vsLld
    Write-Host "Using VS LLVM ld.lld: $linker"
} elseif ($mingwRoots) {
    $mingwLld = Get-ChildItem $mingwRoots[0].FullName -Filter "ld.lld.exe" -Recurse -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if ($mingwLld) {
        $linker = $mingwLld.FullName
        Write-Host "Using LLVM-MinGW ld.lld: $linker"
    }
}

if (-not $linker) {
    # Last resort: copy rust-lld (often WDAC-blocked on Windows).
    $tools = Join-Path $Root "tools"
    New-Item -ItemType Directory -Force -Path $tools | Out-Null
    $candidates = Get-ChildItem "$env:USERPROFILE\.rustup\toolchains" -Recurse -Filter "rust-lld.exe" -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match "x86_64-pc-windows-msvc\\bin\\rust-lld\.exe$" }
    if (-not $candidates) {
        throw "No usable linker found (VS LLVM, LLVM-MinGW, or rust-lld)."
    }
    $dst = Join-Path $tools "lld.exe"
    Copy-Item -Force $candidates[0].FullName $dst
    $linker = $dst
    Write-Host "Copied rust-lld -> $dst (may be WDAC-blocked)"
}

$linkerEscaped = $linker -replace '\\', '\\'
$config = @"
[target.aarch64-unknown-none]
linker = "$linkerEscaped"
rustflags = ["-C", "link-arg=-T$ldScript"]
"@

$configPath = Join-Path $configDir "config.toml"
Set-Content -Path $configPath -Value $config -Encoding UTF8
Write-Host "Updated $configPath"

# Do not put aarch64 linker flags in the workspace .cargo/config.toml —
# guest EL0 builds inherit that file and must use userspace/guest/user.ld.
Write-Host "Now run: .\scripts\build-kernel.ps1"
