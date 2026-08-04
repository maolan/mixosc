; MixOSC Installer
; Run with: makensis.exe installer.nsi
; Requires all binaries and DLLs to be staged in C:\maolan-staging\mixosc

Unicode true

!include "MUI2.nsh"
!include "LogicLib.nsh"

!ifndef MIXOSC_VERSION
!define MIXOSC_VERSION "0.0.11"
!endif

!ifndef MIXOSC_PRODUCT_VERSION
!define MIXOSC_PRODUCT_VERSION "${MIXOSC_VERSION}.0"
!endif

;--------------------------------
; General
;--------------------------------
Name "MixOSC"
OutFile "mixosc-setup.exe"
InstallDir "$LOCALAPPDATA\MixOSC"
InstallDirRegKey HKCU "Software\MixOSC" "InstallDir"
RequestExecutionLevel user

;--------------------------------
; Version Info
;--------------------------------
VIProductVersion "${MIXOSC_PRODUCT_VERSION}"
VIAddVersionKey "ProductName" "MixOSC"
VIAddVersionKey "ProductVersion" "${MIXOSC_VERSION}"
VIAddVersionKey "FileVersion" "${MIXOSC_VERSION}"
VIAddVersionKey "FileDescription" "MixOSC - OSC Mixer Control Surface"
VIAddVersionKey "LegalCopyright" "BSD-2-Clause"

;--------------------------------
; Interface Settings
;--------------------------------
!define MUI_ABORTWARNING
!ifndef MIXOSC_ICON
!define MIXOSC_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!endif
!define MUI_ICON "${MIXOSC_ICON}"
!define MUI_UNICON "${MIXOSC_ICON}"

;--------------------------------
; Pages
;--------------------------------
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

;--------------------------------
; Languages
;--------------------------------
!insertmacro MUI_LANGUAGE "English"

;--------------------------------
; Installer Sections
;--------------------------------
Section "Install"
    SetOutPath "$INSTDIR"

    ; Copy all staged binaries and DLLs
    File "C:\maolan-staging\mixosc\*.*"

    ; Run VC++ Redistributable installer
    ExecWait '"$INSTDIR\vc_redist.x64.exe" /install /quiet /norestart' $0
    Delete "$INSTDIR\vc_redist.x64.exe"

    ; Store installation folder
    WriteRegStr HKCU "Software\MixOSC" "InstallDir" $INSTDIR

    ; Create uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Add to Add/Remove Programs
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC" \
        "DisplayName" "MixOSC"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC" \
        "UninstallString" "$\"$INSTDIR\Uninstall.exe$\""
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC" \
        "DisplayVersion" "${MIXOSC_VERSION}"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC" \
        "Publisher" "Maolan Team"
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC" \
        "NoModify" 1
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC" \
        "NoRepair" 1

    ; Create Start Menu shortcuts
    CreateDirectory "$SMPROGRAMS\MixOSC"
    CreateShortcut "$SMPROGRAMS\MixOSC\MixOSC.lnk" "$INSTDIR\mixosc.exe" "" "$INSTDIR\mixosc.exe" 0
    CreateShortcut "$SMPROGRAMS\MixOSC\Uninstall.lnk" "$INSTDIR\Uninstall.exe" "" "$INSTDIR\Uninstall.exe" 0

    ; Create desktop shortcut
    CreateShortcut "$DESKTOP\MixOSC.lnk" "$INSTDIR\mixosc.exe" "" "$INSTDIR\mixosc.exe" 0
SectionEnd

;--------------------------------
; Uninstaller Section
;--------------------------------
Section "Uninstall"
    Delete "$INSTDIR\mixosc.exe"
    Delete "$INSTDIR\Uninstall.exe"

    Delete "$SMPROGRAMS\MixOSC\MixOSC.lnk"
    Delete "$SMPROGRAMS\MixOSC\Uninstall.lnk"
    RMDir "$SMPROGRAMS\MixOSC"

    Delete "$DESKTOP\MixOSC.lnk"

    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC"
    DeleteRegKey HKCU "Software\MixOSC"

    RMDir "$INSTDIR"
SectionEnd
