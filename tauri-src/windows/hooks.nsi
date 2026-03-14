!addincludedir "${__FILEDIR__}"
!include "nsDialogs.nsh"
!include "StrFunc.nsh"
${StrStr}
${StrRep}
!include "WinMessages.nsh"

!define BLPRNT_BUN_DIR "$LOCALAPPDATA\\{{productName}}\\bun"
!define BLPRNT_BUN_SOURCE "${__FILEDIR__}\..\binaries\bun-x86_64-pc-windows-msvc.exe"

Var BunInstallCheckbox
Var BunInstallChoice
Var BunUninstallCheckbox
Var BunUninstallChoice

Page custom BunInstallPageCreate BunInstallPageLeave
UninstPage custom un.BunUninstallPageCreate un.BunUninstallPageLeave

Function BunInstallPageCreate
  IfSilent bun_install_page_skip
  ${GetOptions} $CMDLINE "/P" $0
  ${IfNot} ${Errors}
    Abort
  ${EndIf}

  !insertmacro MUI_HEADER_TEXT "Optional Runtime" "Choose whether to install Bun"
  nsDialogs::Create 1018
  Pop $0
  ${IfThen} $(^RTL) = 1 ${|} nsDialogs::SetRTL $(^RTL) ${|}

  ${NSD_CreateLabel} 0 0 100% 28u "blprnt can install Bun as a JavaScript runtime for some tools and workflows."
  Pop $1

  ${NSD_CreateCheckbox} 0 42u 100% 10u "Install Bun and add it to my user PATH"
  Pop $BunInstallCheckbox

  StrCmp $BunInstallChoice "" 0 bun_install_page_restore
  StrCpy $BunInstallChoice ${BST_CHECKED}

bun_install_page_restore:
  SendMessage $BunInstallCheckbox ${BM_SETCHECK} $BunInstallChoice 0
  nsDialogs::Show
  Return

bun_install_page_skip:
  Abort
FunctionEnd

Function BunInstallPageLeave
  StrCmp $BunInstallCheckbox "" bun_install_page_leave_done
  ${NSD_GetState} $BunInstallCheckbox $BunInstallChoice

bun_install_page_leave_done:
FunctionEnd

Function un.BunUninstallPageCreate
  IfSilent bun_uninstall_page_skip
  ${GetOptions} $CMDLINE "/P" $0
  ${IfNot} ${Errors}
    Abort
  ${EndIf}

  !insertmacro MUI_HEADER_TEXT "Optional Cleanup" "Choose whether to remove Bun"
  nsDialogs::Create 1018
  Pop $0
  ${IfThen} $(^RTL) = 1 ${|} nsDialogs::SetRTL $(^RTL) ${|}

  ${NSD_CreateLabel} 0 0 100% 28u "blprnt can remove the Bun runtime it installed and clean up your user PATH entry."
  Pop $1

  ${NSD_CreateCheckbox} 0 42u 100% 10u "Remove Bun installed by blprnt"
  Pop $BunUninstallCheckbox

  StrCmp $BunUninstallChoice "" 0 bun_uninstall_page_restore
  IfFileExists "${BLPRNT_BUN_DIR}\bun.exe" 0 bun_uninstall_page_default_off
  StrCpy $BunUninstallChoice ${BST_CHECKED}
  Goto bun_uninstall_page_restore

bun_uninstall_page_default_off:
  StrCpy $BunUninstallChoice ${BST_UNCHECKED}

bun_uninstall_page_restore:
  SendMessage $BunUninstallCheckbox ${BM_SETCHECK} $BunUninstallChoice 0
  nsDialogs::Show
  Return

bun_uninstall_page_skip:
  Abort
FunctionEnd

Function un.BunUninstallPageLeave
  StrCmp $BunUninstallCheckbox "" bun_uninstall_page_leave_done
  ${NSD_GetState} $BunUninstallCheckbox $BunUninstallChoice

bun_uninstall_page_leave_done:
FunctionEnd

Function AddToUserPath
  Exch $0
  Push $1
  Push $2
  Push $3

  ReadRegStr $1 HKCU "Environment" "PATH"
  StrCpy $2 ";$1;"
  ${StrStr} $3 $2 ";$0;"
  StrCmp $3 "" 0 add_to_user_path_done

  StrCmp $1 "" 0 add_to_user_path_append
  StrCpy $1 "$0"
  Goto add_to_user_path_write

add_to_user_path_append:
  StrCpy $1 "$1;$0"

add_to_user_path_write:
  WriteRegExpandStr HKCU "Environment" "PATH" "$1"

add_to_user_path_done:
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

Function RemoveFromUserPath
  Exch $0
  Push $1
  Push $2
  Push $3

  ReadRegStr $1 HKCU "Environment" "PATH"
  StrCmp $1 "" remove_from_user_path_done
  StrCpy $2 ";$1;"

remove_from_user_path_loop:
  ${StrStr} $3 $2 ";$0;"
  StrCmp $3 "" remove_from_user_path_collapse
  ${StrRep} $2 $2 ";$0;" ";"
  Goto remove_from_user_path_loop

remove_from_user_path_collapse:
  ${StrRep} $2 $2 ";;" ";"
  ${StrStr} $3 $2 ";;"
  StrCmp $3 "" remove_from_user_path_trim_leading
  Goto remove_from_user_path_collapse

remove_from_user_path_trim_leading:
  StrCpy $3 $2 1
  StrCmp $3 ";" 0 remove_from_user_path_trim_trailing
  StrCpy $2 $2 "" 1

remove_from_user_path_trim_trailing:
  StrLen $3 $2
  IntCmp $3 0 remove_from_user_path_write_empty
  IntOp $3 $3 - 1
  StrCpy $1 $2 1 $3
  StrCmp $1 ";" 0 remove_from_user_path_write
  StrCpy $2 $2 $3
  Goto remove_from_user_path_write

remove_from_user_path_write_empty:
  StrCpy $2 ""

remove_from_user_path_write:
  WriteRegExpandStr HKCU "Environment" "PATH" "$2"

remove_from_user_path_done:
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

; runs after files are copied
Section "postInstall"
  StrCmp $BunInstallChoice ${BST_CHECKED} bun_install bun_done
bun_install:
  SetOutPath "${BLPRNT_BUN_DIR}"
  File /oname=bun.exe "${BLPRNT_BUN_SOURCE}"
  Push "${BLPRNT_BUN_DIR}"
  Call AddToUserPath
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
bun_done:
SectionEnd

; runs before uninstall
Section "preUninstall"
  ; optional: delete app data you own
  StrCmp $BunUninstallChoice ${BST_CHECKED} bun_remove bun_keep
bun_remove:
  ; Remove PATH entries (current + legacy) and delete directories
  Push "${BLPRNT_BUN_DIR}"
  Call RemoveFromUserPath
  Push "$INSTDIR\\bun"
  Call RemoveFromUserPath
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
  RMDir /r "${BLPRNT_BUN_DIR}"
  RMDir /r "$INSTDIR\\bun"
bun_keep:
  RMDir /r "$APPDATA\{{bundleIdentifier}}"
SectionEnd
