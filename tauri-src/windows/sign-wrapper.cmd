@echo off
setlocal

REM Wrapper to call the PowerShell signing script in a way that Tauri can execute reliably.
REM %1 = path to the binary that Tauri wants to sign.

set "TARGET=%~1"

REM Resolve the directory of this script (windows\)
set "SCRIPT_DIR=%~dp0"

REM Call PowerShell to run sign.ps1 with the target path
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%sign.ps1" "%TARGET%"

endlocal


