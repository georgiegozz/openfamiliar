#Requires -Version 5.1
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

Write-Host "OpenFamiliar bootstrap"
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
    npm install -g pnpm
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "Rust/Cargo not found. Install from https://rustup.rs and re-open the shell."
}

# Prefer MSVC developer environment when available (needs Windows SDK for kernel32.lib)
$vcvars = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
if (-not (Test-Path $vcvars)) {
    $vcvars = "${env:ProgramFiles}\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
}

pnpm install

if (Test-Path $vcvars) {
    Write-Host "Using $vcvars"
    cmd /c "`"$vcvars`" && cargo build --workspace --exclude openfamiliar-desktop"
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "cargo build failed. Ensure Windows 10/11 SDK is installed with VS Build Tools (kernel32.lib)."
    }
}
else {
    Write-Warning "vcvars64.bat not found. Install VS Build Tools 2022 + C++ + Windows SDK, then re-run."
    cargo build --workspace --exclude openfamiliar-desktop
}

Write-Host "Done. JS: pnpm --filter @openfamiliar/desktop dev"
Write-Host "Tauri: pnpm --filter @openfamiliar/desktop exec tauri dev (after SDK works)"
