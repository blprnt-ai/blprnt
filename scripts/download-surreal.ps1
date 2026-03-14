#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
Set-StrictMode -Version Latest

$version = "3.0.0"
$baseUrl = "https://github.com/surrealdb/surrealdb/releases/download/v$version"
$target = "x86_64-pc-windows-msvc"
$archive = "surreal-v$version.windows-amd64.exe"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$outDir = Join-Path $repoRoot "tauri-src/binaries"
New-Item -Path $outDir -ItemType Directory -Force | Out-Null

$tempFile = Join-Path ([System.IO.Path]::GetTempPath()) $archive
$destination = Join-Path $outDir "surreal-$target.exe"

Write-Host "Downloading $archive..."
Invoke-WebRequest -Uri "$baseUrl/$archive" -OutFile $tempFile

Write-Host "Moving $tempFile to $destination..."
Move-Item -Path $tempFile -Destination $destination -Force

Write-Host "  -> surreal-$target.exe"
Write-Host "Done!"
