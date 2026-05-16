#!/usr/bin/env pwsh
# Launch FlareDeck in dev mode on Windows.
#
# Architecture:
#   - Vite runs inside WSL Ubuntu (the node_modules tree was installed there;
#     Windows npm cannot resolve the symlinks in node_modules/.bin).
#   - Tauri dev runs from Windows so cargo targets MSVC (matches dist-windows/).
#   - We override beforeDevCommand via a temp config file (inline -c JSON is
#     mangled by PS 5.1 native-arg quoting).
#
# Prerequisites:
#   - Global tauri CLI: npm install -g @tauri-apps/cli@^2.11
#   - Rust with x86_64-pc-windows-msvc target (default)
#   - WSL Ubuntu with Node 22 via nvm at /home/milzam/.nvm/versions/node/v22.22.2
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts\desktop-dev.ps1
#   (Ctrl+C stops Tauri; close the separate Vite WSL window to stop Vite.)

$ErrorActionPreference = 'Stop'

$ProjectRoot = (Resolve-Path "$PSScriptRoot\..").ProviderPath
$TargetDir   = Join-Path $env:USERPROFILE '.cargo\target-flaredeck'
$OverridePath = Join-Path $env:TEMP 'flaredeck-tauri-override.json'
$SafeCwd     = $env:USERPROFILE

Set-Content -Path $OverridePath -Encoding utf8 -Value '{"build":{"beforeDevCommand":""}}'

Write-Host "==> Project root: $ProjectRoot"
Write-Host "==> Starting Vite in a separate WSL window..."
Start-Process -FilePath 'wsl.exe' -WorkingDirectory $SafeCwd -ArgumentList @(
    '-d', 'Ubuntu', '--',
    'bash', '/home/milzam/flaredeck/scripts/wsl-vite.sh'
) -WindowStyle Normal | Out-Null

Write-Host "==> Waiting for Vite on http://localhost:5173 ..."
$ready = $false
for ($i = 0; $i -lt 60; $i++) {
    try {
        $r = Invoke-WebRequest -Uri 'http://localhost:5173/' -UseBasicParsing -TimeoutSec 2 -ErrorAction Stop
        if ($r.StatusCode -eq 200) { $ready = $true; break }
    } catch {
        Start-Sleep -Seconds 1
    }
}
if (-not $ready) {
    Write-Error "Vite did not become ready on :5173 within 60s. Check the WSL window for errors."
    exit 1
}
Write-Host "==> Vite ready."

$env:CARGO_TARGET_DIR = $TargetDir
$env:CARGO_INCREMENTAL = '0'

Write-Host "==> Launching tauri dev (CARGO_TARGET_DIR=$TargetDir)"
Push-Location $ProjectRoot
try {
    & tauri dev --no-dev-server-wait -c $OverridePath
} finally {
    Pop-Location
}
