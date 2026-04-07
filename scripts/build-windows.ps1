#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
Set-StrictMode -Version Latest

function Assert-Contains {
    param(
        [string]$Path,
        [string]$Pattern
    )

    $content = Get-Content -Path $Path -Raw
    if (-not $content.Contains($Pattern)) {
        throw "Missing expected reference to '$Pattern' in $Path"
    }
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$cargoTomlPath = Join-Path $repoRoot "backend/crates/blprnt/Cargo.toml"
$workflowPath = Join-Path $repoRoot ".github/workflows/release.yml"
$readmePath = Join-Path $repoRoot "README.md"
$targetTriple = "x86_64-pc-windows-msvc"

$cargoToml = Get-Content -Path $cargoTomlPath -Raw
$versionMatch = [regex]::Match($cargoToml, '(?m)^version\s*=\s*"([^"]+)"')
if (-not $versionMatch.Success) {
    throw "Missing version in $cargoTomlPath"
}

$currentVersion = $versionMatch.Groups[1].Value
$releaseStem = "blprnt-v$currentVersion-windows-x86_64"
$binDir = Join-Path $repoRoot "bin"
$packageDir = Join-Path $binDir $releaseStem
$archivePath = Join-Path $binDir "$releaseStem.zip"

Assert-Contains -Path $workflowPath -Pattern "-p blprnt"
Assert-Contains -Path $workflowPath -Pattern "blprnt.exe"
Assert-Contains -Path $readmePath -Pattern "pwsh ./scripts/build-windows.ps1"

Write-Host "Building blprnt v$currentVersion for Windows ($targetTriple)..."

Push-Location $repoRoot
try {
    pnpm --dir frontend install --frozen-lockfile
    pnpm --dir frontend build
    cargo fetch --locked --manifest-path backend/Cargo.toml
    cargo build --release --locked --manifest-path backend/Cargo.toml -p blprnt --target $targetTriple

    if (Test-Path $packageDir) {
        Remove-Item -Path $packageDir -Recurse -Force
    }
    if (Test-Path $archivePath) {
        Remove-Item -Path $archivePath -Force
    }

    New-Item -Path $packageDir -ItemType Directory -Force | Out-Null
    Copy-Item "backend/target/$targetTriple/release/blprnt.exe" "$packageDir/blprnt.exe"
    Copy-Item "frontend/dist" "$packageDir/dist" -Recurse
    & "$repoRoot/scripts/fetch-ripgrep.ps1" -OutputPath "$packageDir/tools/rg.exe"
    Copy-Item "README.md","LICENSE" $packageDir

    Compress-Archive -Path "$packageDir/*" -DestinationPath $archivePath -Force
}
finally {
    Pop-Location
}

Write-Host "Packaged $archivePath"
