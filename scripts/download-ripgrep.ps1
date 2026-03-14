#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
Set-StrictMode -Version Latest

$version = "15.1.0"
$baseUrl = "https://github.com/BurntSushi/ripgrep/releases/download/$version"
$target = "ripgrep-$version-x86_64-pc-windows-msvc"
$archive = "$target.zip"
$targetName = "rg-x86_64-pc-windows-msvc.exe"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$outDir = Join-Path $repoRoot "tauri-src/binaries"
New-Item -Path $outDir -ItemType Directory -Force | Out-Null

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("ripgrep-" + [System.Guid]::NewGuid().ToString("N"))
$archivePath = Join-Path $tempRoot $archive
$extractDir = Join-Path $tempRoot "extract"

try {
    New-Item -Path $tempRoot -ItemType Directory -Force | Out-Null
    New-Item -Path $extractDir -ItemType Directory -Force | Out-Null

    Write-Host "Downloading $archive..."
    Invoke-WebRequest -Uri "$baseUrl/$archive" -OutFile $archivePath

    Write-Host "Extracting $archive..."
    Expand-Archive -Path $archivePath -DestinationPath $extractDir -Force

    $sourcePath = Join-Path (Join-Path $extractDir $target) "rg.exe"
    $destination = Join-Path $outDir $targetName

    Write-Host "Moving $sourcePath to $destination..."
    Move-Item -Path $sourcePath -Destination $destination -Force
}
finally {
    if (Test-Path $tempRoot) {
        Remove-Item -Path $tempRoot -Recurse -Force
    }
}

Write-Host "  -> ripgrep-x86_64-pc-windows-msvc"
Write-Host "Done!"
