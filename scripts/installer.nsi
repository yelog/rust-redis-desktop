!include "MUI2.nsh"
!include "LogicLib.nsh"

!define APP_NAME "Rust Redis Desktop"
!define APP_VERSION "${VERSION}"
!define APP_PUBLISHER "yelog"
!define APP_URL "https://github.com/yelog/rust-redis-desktop"
!define APP_EXE "rust-redis-desktop.exe"
!define APP_GUID "D5A5B5C5-1234-5678-9ABC-DEF012345678"
!define WEBVIEW2_GUID "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
!define WEBVIEW2_DOWNLOAD_URL "https://developer.microsoft.com/en-us/microsoft-edge/webview2/"

Name "${APP_NAME} ${APP_VERSION}"
OutFile "rust-redis-desktop-setup-${APP_VERSION}.exe"
InstallDir "$LOCALAPPDATA\Programs\${APP_NAME}"
InstallDirRegKey HKCU "Software\${APP_NAME}" "Install_Dir"
RequestExecutionLevel user
Unicode True

!define MUI_ABORTWARNING
!define MUI_ICON "icon.ico"
!define MUI_UNICON "icon.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE.txt"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "SimpChinese"

LangString APP_DESC ${LANG_ENGLISH} "A Redis desktop manager written in Rust"
LangString APP_DESC ${LANG_SIMPCHINESE} "使用 Rust 编写的 Redis 桌面管理器"

VIProductVersion "${VERSION}.0"
VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductName" "${APP_NAME}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "Comments" "${APP_URL}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "CompanyName" "${APP_PUBLISHER}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "LegalCopyright" "Copyright (c) ${APP_PUBLISHER}"
VIAddVersionKey /LANG=${LANG_ENGLISH} "FileDescription" "${APP_NAME} Installer"
VIAddVersionKey /LANG=${LANG_ENGLISH} "FileVersion" "${APP_VERSION}"

Section "Main Section" SecMain
    SectionIn RO
    
    SetOutPath "$INSTDIR"
    
    File /r "app\*.*"
    File "icon.ico"
    File "MicrosoftEdgeWebview2Setup.exe"
    
    StrCpy $0 "${APP_EXE}"
    nsExec::ExecToStack 'taskkill /IM "$0" /F'
    Pop $1
    Pop $2

    ReadRegStr $3 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\${WEBVIEW2_GUID}" "pv"
    ${If} $3 == ""
        ReadRegStr $3 HKCU "Software\Microsoft\EdgeUpdate\Clients\${WEBVIEW2_GUID}" "pv"
    ${EndIf}

    ${If} $3 == ""
    ${OrIf} $3 == "0.0.0.0"
        DetailPrint "Installing Microsoft Edge WebView2 Runtime..."
        nsExec::ExecToStack '"$INSTDIR\MicrosoftEdgeWebview2Setup.exe" /silent /install'
        Pop $4
        Pop $5

        ReadRegStr $3 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\${WEBVIEW2_GUID}" "pv"
        ${If} $3 == ""
            ReadRegStr $3 HKCU "Software\Microsoft\EdgeUpdate\Clients\${WEBVIEW2_GUID}" "pv"
        ${EndIf}

        ${If} $4 != "0"
        ${OrIf} $3 == ""
        ${OrIf} $3 == "0.0.0.0"
            Delete "$INSTDIR\MicrosoftEdgeWebview2Setup.exe"
            MessageBox MB_ICONSTOP|MB_OK "Failed to install Microsoft Edge WebView2 Runtime.$\r$\nExit code: $4$\r$\n$\r$\nPlease install it manually from:$\r$\n${WEBVIEW2_DOWNLOAD_URL}"
            Abort
        ${EndIf}
    ${EndIf}

    Delete "$INSTDIR\MicrosoftEdgeWebview2Setup.exe"
    
    WriteRegStr HKCU "Software\${APP_NAME}" "Install_Dir" "$INSTDIR"
    WriteRegStr HKCU "Software\${APP_NAME}" "Version" "${APP_VERSION}"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayName" "${APP_NAME}"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "UninstallString" '"$INSTDIR\uninstall.exe"'
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayIcon" "$INSTDIR\icon.ico"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayVersion" "${APP_VERSION}"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "Publisher" "${APP_PUBLISHER}"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "URLInfoAbout" "${APP_URL}"
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "NoModify" 1
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "NoRepair" 1
    
    WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Desktop Shortcut" SecDesktop
    CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\icon.ico"
SectionEnd

Section "Start Menu Shortcuts" SecStartMenu
    CreateDirectory "$SMPROGRAMS\${APP_NAME}"
    CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}" "" "$INSTDIR\icon.ico"
    CreateShortcut "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk" "$INSTDIR\uninstall.exe" "" "$INSTDIR\uninstall.exe"
SectionEnd

LangString DESC_SecMain ${LANG_ENGLISH} "Install ${APP_NAME} (required)"
LangString DESC_SecMain ${LANG_SIMPCHINESE} "安装 ${APP_NAME}（必需）"
LangString DESC_SecDesktop ${LANG_ENGLISH} "Create a desktop shortcut"
LangString DESC_SecDesktop ${LANG_SIMPCHINESE} "创建桌面快捷方式"
LangString DESC_SecStartMenu ${LANG_ENGLISH} "Create start menu shortcuts"
LangString DESC_SecStartMenu ${LANG_SIMPCHINESE} "创建开始菜单快捷方式"

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SecMain} $(DESC_SecMain)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} $(DESC_SecDesktop)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecStartMenu} $(DESC_SecStartMenu)
!insertmacro MUI_FUNCTION_DESCRIPTION_END

Section "Uninstall"
    nsExec::ExecToStack 'taskkill /IM "${APP_EXE}" /F'
    Pop $0
    Pop $1
    
    Delete "$INSTDIR\*.*"
    RMDir /r "$INSTDIR"
    
    Delete "$DESKTOP\${APP_NAME}.lnk"
    Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
    Delete "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk"
    RMDir "$SMPROGRAMS\${APP_NAME}"
    
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
    DeleteRegKey HKCU "Software\${APP_NAME}"
SectionEnd
