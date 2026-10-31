# Builds a signed installer and publishes it for the auto-updater.
# Usage: .\scripts\release.ps1 -Version 0.2.0 [-Notes "O que mudou"] [-BuildOnly]
param(
    [Parameter(Mandatory)][ValidatePattern('^\d+\.\d+\.\d+$')][string]$Version,
    [string]$Notes = "Atualização do Sharezynho",
    [string]$ReleaseRepo = "starzynhobr/sharezynho2",
    [string]$KeyPath = "$HOME\.tauri\sharezynho.key",
    [switch]$BuildOnly
)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root

function Set-FileVersion($Path, $Pattern, $Replacement) {
    $content = [IO.File]::ReadAllText($Path)
    $regex = [regex]$Pattern
    [IO.File]::WriteAllText($Path, $regex.Replace($content, $Replacement, 1))
}

Set-FileVersion "$root\package.json" '"version": "[^"]+"' "`"version`": `"$Version`""
Set-FileVersion "$root\src-tauri\tauri.conf.json" '"version": "[^"]+"' "`"version`": `"$Version`""
Set-FileVersion "$root\src-tauri\Cargo.toml" '(?m)^version = "[^"]+"' "version = `"$Version`""

$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $KeyPath -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = (Get-Content "$KeyPath.password" -Raw).Trim()
npm run tauri build
if ($LASTEXITCODE -ne 0) { throw "Build falhou" }

$bundle = "$root\src-tauri\target\release\bundle\nsis"
$installer = Get-Item "$bundle\Sharezynho_${Version}_x64-setup.exe"
$signature = Get-Content "$($installer.FullName).sig" -Raw

# GitHub replaces spaces in asset names, so the URL uses the file name as is
$url = "https://github.com/$ReleaseRepo/releases/download/v$Version/$($installer.Name)"
$manifest = "$bundle\latest.json"
$json = @{
    version   = $Version
    notes     = $Notes
    pub_date  = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = @{ "windows-x86_64" = @{ signature = $signature.Trim(); url = $url } }
} | ConvertTo-Json -Depth 4
[IO.File]::WriteAllText($manifest, $json)

if ($BuildOnly) {
    Write-Host "Instalador: $($installer.FullName)"
    return
}

gh release create "v$Version" $installer.FullName "$($installer.FullName).sig" $manifest `
    --repo $ReleaseRepo --title "Sharezynho $Version" --notes $Notes
if ($LASTEXITCODE -ne 0) { throw "Publicação falhou" }
Write-Host "Publicado: https://github.com/$ReleaseRepo/releases/tag/v$Version"
