param(
  [Parameter(Mandatory = $true)]
  [string] $TargetFilePath
)

$ErrorActionPreference = 'Stop'

$now = Get-Date -Format "yyyy-MM-dd HH:mm:ss.fff"
$rawTargetFilePath = $TargetFilePath
$TargetFilePath = $TargetFilePath.Trim()
if ($TargetFilePath.StartsWith('"') -and $TargetFilePath.EndsWith('"')) {
  $TargetFilePath = $TargetFilePath.Trim('"')
}

Write-Host "[$now] sign.ps1 start"
Write-Host "[$now] PID: $PID"
Write-Host "[$now] Target (raw): $rawTargetFilePath"
Write-Host "[$now] Target (normalized): $TargetFilePath"

if (-not (Test-Path -LiteralPath $TargetFilePath)) {
  # NSIS sometimes passes a temp file path before it's fully materialized.
  # Wait briefly to allow the file to appear before failing.
  $maxWaitMs = 10000
  $delayMs = 250
  $elapsedMs = 0

  $dirPath = Split-Path -Path $TargetFilePath -Parent
  Write-Host "Target missing, waiting for file to exist: $TargetFilePath"
  Write-Host "Target dir exists: $(Test-Path -LiteralPath $dirPath) ($dirPath)"
  while ($elapsedMs -lt $maxWaitMs -and -not (Test-Path -LiteralPath $TargetFilePath)) {
    Start-Sleep -Milliseconds $delayMs
    $elapsedMs += $delayMs
  }
}

if (-not (Test-Path -LiteralPath $TargetFilePath)) {
  $dirPath = Split-Path -Path $TargetFilePath -Parent
  Write-Host "Final check: target exists = $(Test-Path -LiteralPath $TargetFilePath)"
  Write-Host "Final check: dir exists = $(Test-Path -LiteralPath $dirPath) ($dirPath)"
  throw "File does not exist after waiting: $TargetFilePath"
}

# Required config (normally provided via env vars in CI)
# The endpoint/account/profile can be hard-coded here for your tenant, but
# credentials (AZURE_CLIENT_ID / AZURE_CLIENT_SECRET / AZURE_TENANT_ID / AZURE_SUBSCRIPTION_ID)
# must come from the environment (CircleCI context, local shell, etc.).

$trustedSigningEndpoint = $env:AZURE_TRUSTED_SIGNING_ENDPOINT
if ([string]::IsNullOrWhiteSpace($trustedSigningEndpoint)) {
  # Default to the eus endpoint if not provided explicitly
  $trustedSigningEndpoint = "https://eus.codesigning.azure.net/"
}

$codeSigningAccountName = $env:AZURE_TRUSTED_SIGNING_ACCOUNT_NAME
if ([string]::IsNullOrWhiteSpace($codeSigningAccountName)) {
  $codeSigningAccountName = "blprnt-ai"
}

$certificateProfileName = $env:AZURE_TRUSTED_SIGNING_CERT_PROFILE_NAME
if ([string]::IsNullOrWhiteSpace($certificateProfileName)) {
  $certificateProfileName = "blprnt"
}

if ([string]::IsNullOrWhiteSpace($env:AZURE_CLIENT_ID))       { throw "Missing env var: AZURE_CLIENT_ID" }
if ([string]::IsNullOrWhiteSpace($env:AZURE_CLIENT_SECRET))   { throw "Missing env var: AZURE_CLIENT_SECRET" }
if ([string]::IsNullOrWhiteSpace($env:AZURE_TENANT_ID))       { throw "Missing env var: AZURE_TENANT_ID" }
if ([string]::IsNullOrWhiteSpace($env:AZURE_SUBSCRIPTION_ID)) { throw "Missing env var: AZURE_SUBSCRIPTION_ID" }

# Prefer newest installed Windows 10 SDK x64 signtool
$sdkSigntoolCandidates = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin" -Directory -ErrorAction SilentlyContinue |
  Where-Object { $_.Name -match '^\d+\.\d+\.\d+\.\d+$' } |
  Sort-Object Name -Descending |
  ForEach-Object { Join-Path $_.FullName "x64\signtool.exe" }

$signtoolPath = $sdkSigntoolCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $signtoolPath) {
  throw "Could not find x64 signtool.exe under Windows Kits. Install Windows SDK."
}

# Find Azure.CodeSigning.Dlib.dll (installed by Trusted Signing Client Tools)
# Preferred location is under LOCALAPPDATA per current Microsoft docs:
#   %LOCALAPPDATA%\Microsoft\MicrosoftTrustedSigningClientTools\Azure.CodeSigning.Dlib.dll

$dlibPath = $null

# 1) Check the documented LOCALAPPDATA location first (works both locally and in CI)
$localDlibPath = Join-Path $env:LOCALAPPDATA "Microsoft\MicrosoftTrustedSigningClientTools\Azure.CodeSigning.Dlib.dll"
if ($env:LOCALAPPDATA -and (Test-Path $localDlibPath)) {
  $dlibPath = $localDlibPath
  Write-Host "Found DLL at LOCALAPPDATA: $dlibPath"
} else {
  # 2) Fallback: Search common installation paths
  #    (older or different layouts might place the DLL under Program Files)
  $searchPaths = @(
    "${env:ProgramFiles}\Microsoft",
    "${env:ProgramFiles(x86)}\Microsoft",
    "C:\Program Files\Microsoft",
    "C:\Program Files (x86)\Microsoft",
    "C:\Program Files",
    "C:\Program Files (x86)"
  )

  # Try to find installation path from registry
  try {
    $regPath = Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*" -ErrorAction SilentlyContinue |
      Where-Object { $_.DisplayName -like "*Azure*Trusted*Signing*" -or $_.DisplayName -like "*Trusted*Signing*Client*" } |
      Select-Object -First 1 -ExpandProperty InstallLocation
    
    if ($regPath -and (Test-Path $regPath)) {
      Write-Host "Found installation path from registry: $regPath"
      $searchPaths = @($regPath) + $searchPaths
    }
  } catch {
    # Registry check failed, continue with default paths
  }

  # Also check common .NET installation paths
  $dotnetPaths = @(
    "${env:ProgramFiles}\dotnet",
    "${env:ProgramFiles(x86)}\dotnet"
  )
  foreach ($dotnetPath in $dotnetPaths) {
    if (Test-Path $dotnetPath) {
      $searchPaths += $dotnetPath
    }
  }

  foreach ($searchPath in $searchPaths) {
    if (Test-Path $searchPath) {
      Write-Host "Searching in: $searchPath"
      $found = Get-ChildItem -Path $searchPath -Recurse -Filter "Azure.CodeSigning.Dlib.dll" -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match '\\x64\\' } |
        Select-Object -First 1 -ExpandProperty FullName
      
      if ($found) {
        $dlibPath = $found
        Write-Host "Found DLL at: $dlibPath"
        break
      }
    }
  }
}

if (-not $dlibPath) {
  # Provide helpful debug information
  Write-Host "`nDLL search failed. Debug information:"
  Write-Host "Search paths checked:"
  foreach ($path in $searchPaths) {
    $exists = Test-Path $path
    Write-Host "  $path : $(if ($exists) { 'exists' } else { 'not found' })"
  }
  
  # Try to find any version of the DLL for debugging (search entire C: drive if needed)
  Write-Host "`nSearching for any Azure.CodeSigning.Dlib.dll files..."
  $anyDll = Get-ChildItem -Path "C:\Program Files*" -Recurse -Filter "Azure.CodeSigning.Dlib.dll" -ErrorAction SilentlyContinue | Select-Object -First 10
  if ($anyDll) {
    Write-Host "Found DLL(s) (may not be x64):"
    $anyDll | ForEach-Object { 
      $arch = if ($_.FullName -match '\\x64\\') { 'x64' } elseif ($_.FullName -match '\\x86\\') { 'x86' } else { 'unknown' }
      Write-Host "  [$arch] $($_.FullName)" 
    }
  } else {
    Write-Host "No Azure.CodeSigning.Dlib.dll files found anywhere in Program Files."
    Write-Host "`nThe Azure Trusted Signing Client Tools may not be installed."
    Write-Host "To install, run: .\tauri-src\windows\choco.ps1"
    Write-Host "Or download from: https://download.microsoft.com/download/6d9cb638-4d5f-438d-9f21-23f0f4405944/TrustedSigningClientTools.msi"
  }
  
  throw "Could not find Azure.CodeSigning.Dlib.dll (x64). Install Microsoft.Azure.TrustedSigningClientTools."
}

$metadataDirectory = Join-Path $PSScriptRoot "tmp"
New-Item -ItemType Directory -Path $metadataDirectory -Force | Out-Null
$metadataPath = Join-Path $metadataDirectory "metadata.json"

# Write metadata as UTF-8 WITHOUT BOM.
# The Dlib expects pure UTF-8 JSON; the default PowerShell UTF8 encoding adds a BOM (0xEF),
# which causes the JsonException you saw.
$metadataObject = @{
  Endpoint = $trustedSigningEndpoint
  CodeSigningAccountName = $codeSigningAccountName
  CertificateProfileName = $certificateProfileName
}

$metadataJson = $metadataObject | ConvertTo-Json -Depth 4

# Use .NET directly to control encoding and omit BOM
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($metadataPath, $metadataJson, $utf8NoBom)

# Timestamp server (Microsoft example commonly used)
$timestampUrl = "http://timestamp.acs.microsoft.com"

Write-Host "Signing:  $TargetFilePath"
Write-Host "signtool: $signtoolPath"
Write-Host "dlib:     $dlibPath"
Write-Host "metadata: $metadataPath"

& $signtoolPath sign `
  /v /debug `
  /fd SHA256 `
  /tr $timestampUrl /td SHA256 `
  /d "blprnt" `
  /dlib $dlibPath `
  /dmdf $metadataPath `
  $TargetFilePath

& $signtoolPath verify /pa /v $TargetFilePath
