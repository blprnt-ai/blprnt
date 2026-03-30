#!/usr/bin/env pwsh
param(
    [Parameter(Mandatory = $true)]
    [string]$OutputPath
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
Set-StrictMode -Version Latest

$version = "15.1.0"
$assetStem = "ripgrep-$version-x86_64-pc-windows-msvc"
$archiveName = "$assetStem.zip"
$baseUrl = "https://github.com/BurntSushi/ripgrep/releases/download/$version"
$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("fetch-ripgrep-" + [System.Guid]::NewGuid().ToString("N"))
$archivePath = Join-Path $tempRoot $archiveName
$extractDir = Join-Path $tempRoot "extract"

try {
    New-Item -Path $tempRoot -ItemType Directory -Force | Out-Null
    New-Item -Path $extractDir -ItemType Directory -Force | Out-Null
    New-Item -Path (Split-Path -Parent $OutputPath) -ItemType Directory -Force | Out-Null

    Write-Host "Downloading $archiveName..."
    Invoke-WebRequest -Uri "$baseUrl/$archiveName" -OutFile $archivePath

    Write-Host "Extracting $archiveName..."
    Expand-Archive -Path $archivePath -DestinationPath $extractDir -Force

    $binaryPath = Get-ChildItem -Path $extractDir -Filter "rg.exe" -File -Recurse | Select-Object -First 1 -ExpandProperty FullName
    if (-not $binaryPath) {
        throw "Expected rg.exe in $archiveName, but none was found"
    }

    Move-Item -Path $binaryPath -Destination $OutputPath -Force
}
finally {
    if (Test-Path $tempRoot) {
        Remove-Item -Path $tempRoot -Recurse -Force
    }
}

Write-Host "Bundled ripgrep at $OutputPath"
