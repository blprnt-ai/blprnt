$ErrorActionPreference = 'Stop'

# Ensure Chocolatey is installed
if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
  Write-Host "Installing Chocolatey..."
  Set-ExecutionPolicy Bypass -Scope Process -Force
  [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
  Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
}

# Windows SDK (includes signtool)
choco install windows-sdk-10-version-2004-all -y

# .NET 8 runtime (required by Trusted Signing)
choco install dotnet-8.0-runtime -y

# Azure CLI
choco install azure-cli -y

# Azure Trusted Signing Client Tools
# Download to a system-accessible temp location (C:\Windows\Temp) so the Windows Installer
# service (running as SYSTEM) has permission to read the MSI. User profile temp directories
# can cause 2203/1620 errors because SYSTEM cannot access them.
$tempDir = "C:\Windows\Temp"
if (-not (Test-Path $tempDir)) {
  New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
}

$msiPath = Join-Path $tempDir "TrustedSigningClientTools.msi"

# Direct download URL from Microsoft (more reliable than aka.ms short link)
$msiUrl = "https://download.microsoft.com/download/6d9cb638-4d5f-438d-9f21-23f0f4405944/TrustedSigningClientTools.msi"

Write-Host "Downloading Azure Trusted Signing Client Tools from: $msiUrl"
Write-Host "Saving to: $msiPath"

try {
  Invoke-WebRequest `
    -Uri $msiUrl `
    -OutFile $msiPath `
    -UseBasicParsing
} catch {
  Write-Host "Download failed: $_"
  throw "Failed to download Azure Trusted Signing Client Tools MSI"
}

# Verify the MSI file is valid (not an HTML redirect or corrupted)
$msiFile = Get-Item $msiPath -ErrorAction Stop
Write-Host "Downloaded MSI: $($msiFile.Length) bytes"

if ($msiFile.Length -lt 1024) {
  Write-Host "Error: MSI file seems too small ($($msiFile.Length) bytes). Checking contents..."
  $firstBytes = Get-Content $msiPath -TotalCount 10 -Raw
  if ($firstBytes -match '<html|<!DOCTYPE|HTTP|Error') {
    Write-Host "Error: Downloaded file appears to be HTML, not an MSI. Content preview:"
    Get-Content $msiPath -TotalCount 30
    throw "Downloaded file is not a valid MSI package"
  }
  throw "Downloaded file is too small to be a valid MSI"
}

# Verify MSI signature/structure by checking for MSI magic bytes
$msiBytes = [System.IO.File]::ReadAllBytes($msiPath)
if ($msiBytes.Length -lt 8) {
  throw "MSI file is too small"
}
$msiHeader = [System.Text.Encoding]::ASCII.GetString($msiBytes[0..7])
if ($msiHeader -ne "MSI`0`0`0`0`0`0") {
  Write-Host "Warning: File does not appear to be a valid MSI (header: $msiHeader)"
  Write-Host "Proceeding anyway, but installation may fail..."
} else {
  Write-Host "MSI file validation passed (valid MSI header)"
}

Write-Host "Installing Azure Trusted Signing Client Tools from: $msiPath"
$installLog = Join-Path $tempDir "trusted-signing-install.log"

# Ensure the file is readable by SYSTEM before installation
$acl = Get-Acl $msiPath
$accessRule = New-Object System.Security.AccessControl.FileSystemAccessRule(
  "SYSTEM",
  "Read",
  "Allow"
)
$acl.SetAccessRule($accessRule)
Set-Acl -Path $msiPath -AclObject $acl

$process = Start-Process msiexec.exe `
  -ArgumentList "/i `"$msiPath`" /qn /l*v `"$installLog`"" `
  -Wait `
  -PassThru `
  -NoNewWindow

if ($process.ExitCode -ne 0) {
  Write-Host "MSI installation failed with exit code $($process.ExitCode)"
  Write-Host "MSI path: $msiPath"
  Write-Host "MSI exists: $(Test-Path $msiPath)"
  Write-Host "MSI size: $((Get-Item $msiPath).Length) bytes"
  if (Test-Path $installLog) {
    Write-Host "Installation log (last 100 lines):"
    Get-Content $installLog -Tail 100
  }
  throw "Failed to install Azure Trusted Signing Client Tools (exit code: $($process.ExitCode))"
}

Write-Host "Installation completed. Verifying DLL location..."

# Wait a moment for file system to sync
Start-Sleep -Seconds 2

# Verify the DLL exists
$dllPaths = @(
  "C:\Program Files",
  "C:\Program Files (x86)",
  "${env:ProgramFiles}\Microsoft",
  "${env:ProgramFiles(x86)}\Microsoft"
)

$foundDll = $null
foreach ($path in $dllPaths) {
  if (Test-Path $path) {
    $dll = Get-ChildItem -Path $path -Recurse -Filter "Azure.CodeSigning.Dlib.dll" -ErrorAction SilentlyContinue |
      Where-Object { $_.FullName -match '\\x64\\' } |
      Select-Object -First 1
    if ($dll) {
      $foundDll = $dll.FullName
      Write-Host "Found DLL at: $foundDll"
      break
    }
  }
}

if (-not $foundDll) {
  Write-Host "Warning: DLL not found immediately after installation. It may be available after PATH refresh."
  # List all found DLLs for debugging
  $allDlls = Get-ChildItem -Path "C:\Program Files*" -Recurse -Filter "Azure.CodeSigning.Dlib.dll" -ErrorAction SilentlyContinue
  if ($allDlls) {
    Write-Host "Found DLL(s) (may not be x64):"
    $allDlls | ForEach-Object { Write-Host "  $($_.FullName)" }
  }
}