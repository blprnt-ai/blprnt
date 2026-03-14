#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
Set-StrictMode -Version Latest

$version = "0.1.0-alpha.1743007075"
$baseUrl = "https://github.com/getgrit/gritql/releases/download/v$version"
$target = "x86_64-pc-windows-msvc"
$archive = "grit-$target.tar.gz"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$outDir = Join-Path $repoRoot "tauri-src/binaries"
New-Item -Path $outDir -ItemType Directory -Force | Out-Null

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("grit-" + [System.Guid]::NewGuid().ToString("N"))
$archivePath = Join-Path $tempRoot $archive
$extractDir = Join-Path $tempRoot "extract"

try {
    New-Item -Path $tempRoot -ItemType Directory -Force | Out-Null
    New-Item -Path $extractDir -ItemType Directory -Force | Out-Null

    Write-Host "Downloading $archive..."
    Invoke-WebRequest -Uri "$baseUrl/$archive" -OutFile $archivePath

    Write-Host "Extracting $archive..."
    tar -xzf $archivePath -C $extractDir
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to extract $archive"
    }

    $sourcePath = Join-Path (Join-Path $extractDir "grit-$target") "grit.exe"
    $destination = Join-Path $outDir "grit-$target.exe"

    Write-Host "Moving $sourcePath to $destination..."
    Move-Item -Path $sourcePath -Destination $destination -Force
}
finally {
    if (Test-Path $tempRoot) {
        Remove-Item -Path $tempRoot -Recurse -Force
    }
}

Write-Host "  -> grit-$target"
Write-Host "Done!"
