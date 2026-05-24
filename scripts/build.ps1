#Requires -Version 5.1
$ErrorActionPreference = "Stop"

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
$target    = "x86_64-pc-windows-msvc"
$targetDir = "C:\cargo-target"
$nsisPath  = "C:\nsis-3.10\makensis.exe"
$staging   = "C:\maolan-staging\mixosc"

# ---------------------------------------------------------------------------
# Version from Cargo.toml
# ---------------------------------------------------------------------------
$cargoToml = Join-Path (Split-Path $PSScriptRoot -Parent) "Cargo.toml"
$pkgVersion = "0.0.0"
if (Test-Path $cargoToml) {
    $versionLine = Select-String -Path $cargoToml -Pattern '^version\s*=\s*"(.+)"' | Select-Object -First 1
    if ($versionLine) {
        $pkgVersion = $versionLine.Matches.Groups[1].Value
    }
}
Write-Host "Package version: $pkgVersion"

# ---------------------------------------------------------------------------
# Elevation check
# ---------------------------------------------------------------------------
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Warning "This script is NOT running as Administrator."
    Write-Warning "Some installations (VS Build Tools, NSIS to C:\) may require elevation."
    Write-Host ""
}

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
function Test-Command {
    param([string]$Name)
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Ensure-Git {
    $gitPath = "$env:ProgramFiles\Git\cmd\git.exe"
    if (Test-Path $gitPath) {
        Write-Host "Git already installed at $gitPath"
        $env:PATH = "$env:ProgramFiles\Git\cmd;$env:PATH"
        return
    }
    Write-Host "Installing Git..."
    $installer = "$env:TEMP\Git-installer.exe"
    Invoke-WebRequest -Uri "https://github.com/git-for-windows/git/releases/download/v2.44.0.windows.1/Git-2.44.0-64-bit.exe" -OutFile $installer
    Start-Process -FilePath $installer -ArgumentList "/VERYSILENT","/NORESTART" -Wait
    $env:PATH = "$env:ProgramFiles\Git\cmd;$env:PATH"
}

function Ensure-VSBuildTools {
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $installPath = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if ($installPath) {
            Write-Host "VS Build Tools found at $installPath"
            return
        }
    }
    Write-Host "Installing Visual Studio Build Tools..."
    $installer = "$env:TEMP\vs_buildtools.exe"
    if (-not (Test-Path $installer)) {
        Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_buildtools.exe" -OutFile $installer
    }
    Start-Process -FilePath $installer -ArgumentList `
        "--quiet","--wait","--add","Microsoft.VisualStudio.Workload.VCTools","--add","Microsoft.VisualStudio.Component.Windows11SDK.22621" `
        -Wait
}

function Import-VSEnv {
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) { return }
    $installPath = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $installPath) { return }
    $vcvars = Join-Path $installPath "VC\Auxiliary\Build\vcvars64.bat"
    if (-not (Test-Path $vcvars)) { return }
    $tempFile = [System.IO.Path]::GetTempFileName()
    cmd /c "`"$vcvars`" && set > `"$tempFile`""
    Get-Content $tempFile | ForEach-Object {
        if ($_ -match '^(\w+)=(.*)$') {
            [Environment]::SetEnvironmentVariable($matches[1], $matches[2], 'Process')
        }
    }
    Remove-Item $tempFile
    Write-Host "Imported VS environment"
}

function Ensure-Rust {
    if (Test-Command "cargo") {
        Write-Host "Rust already installed: $(cargo --version)"
        return
    }
    Write-Host "Installing Rust..."
    $installer = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $installer
    Start-Process -FilePath $installer -ArgumentList "-y","--default-toolchain","stable" -Wait
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
}

function Ensure-NSIS {
    if (Test-Path $nsisPath) {
        Write-Host "NSIS already installed at $nsisPath"
        return
    }
    Write-Host "Installing NSIS..."
    $zip = "$env:TEMP\nsis.zip"
    Invoke-WebRequest -Uri "https://sourceforge.net/projects/nsis/files/NSIS%203/3.10/nsis-3.10.zip/download" -OutFile $zip
    Expand-Archive -Path $zip -DestinationPath "C:\nsis-3.10-temp" -Force
    Move-Item "C:\nsis-3.10-temp\nsis-3.10" "C:\nsis-3.10" -Force
    Remove-Item "C:\nsis-3.10-temp" -Recurse -Force
}

# ---------------------------------------------------------------------------
# VC++ Redistributable
# ---------------------------------------------------------------------------
$maolanRoot = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$vcRedist = Join-Path $maolanRoot "vc_redist.x64.exe"
if (-not (Test-Path $vcRedist)) {
    Write-Host "Downloading VC++ Redistributable to $vcRedist..."
    Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vc_redist.x64.exe" -OutFile $vcRedist
}

# ---------------------------------------------------------------------------
# Main flow
# ---------------------------------------------------------------------------
Ensure-VSBuildTools
Import-VSEnv
Ensure-Rust
Ensure-NSIS
Ensure-Git

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
Write-Host "Building mixosc (release)..."
Push-Location (Split-Path $PSScriptRoot -Parent)
cargo build --release --target $target --target-dir $targetDir
Pop-Location

# ---------------------------------------------------------------------------
# Stage
# ---------------------------------------------------------------------------
Write-Host "Staging files to $staging..."
New-Item -ItemType Directory -Force $staging | Out-Null
Copy-Item "$targetDir\$target\release\mixosc.exe" $staging -Force
Copy-Item $vcRedist $staging -Force

# ---------------------------------------------------------------------------
# Installer
# ---------------------------------------------------------------------------
Write-Host "Building installer..."
$nsiTemp = "$env:TEMP\mixosc-installer"
New-Item -ItemType Directory -Force $nsiTemp | Out-Null

$installerNsi = @"
; MixOSC Installer
!include "MUI2.nsh"
!include "LogicLib.nsh"

Name "MixOSC"
OutFile "mixosc-setup.exe"
InstallDir "`$LOCALAPPDATA\MixOSC"
RequestExecutionLevel user

VIProductVersion "$pkgVersion.0"
VIAddVersionKey "ProductName" "MixOSC"
VIAddVersionKey "ProductVersion" "$pkgVersion"
VIAddVersionKey "FileVersion" "$pkgVersion"
VIAddVersionKey "FileDescription" "MixOSC - OSC Mixer Control Surface"
VIAddVersionKey "LegalCopyright" "BSD-2-Clause"

!define MUI_ABORTWARNING
!define MUI_ICON "`${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "`${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Section "Install"
    SetOutPath "`$INSTDIR"
    File "$staging\*.*"
    ExecWait '"`$INSTDIR\vc_redist.x64.exe" /install /quiet /norestart' `$0
    Delete "`$INSTDIR\vc_redist.x64.exe"
    WriteRegStr HKCU "Software\MixOSC" "InstallDir" `$INSTDIR
    WriteUninstaller "`$INSTDIR\Uninstall.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC" "DisplayName" "MixOSC"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC" "UninstallString" "`$`"`$INSTDIR\Uninstall.exe`$`""
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC" "DisplayVersion" "$pkgVersion"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC" "Publisher" "Maolan Team"
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC" "NoModify" 1
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC" "NoRepair" 1
    CreateDirectory "`$SMPROGRAMS\MixOSC"
    CreateShortcut "`$SMPROGRAMS\MixOSC\MixOSC.lnk" "`$INSTDIR\mixosc.exe"
    CreateShortcut "`$SMPROGRAMS\MixOSC\Uninstall.lnk" "`$INSTDIR\Uninstall.exe"
    CreateShortcut "`$DESKTOP\MixOSC.lnk" "`$INSTDIR\mixosc.exe"
SectionEnd

Section "Uninstall"
    Delete "`$INSTDIR\mixosc.exe"
    Delete "`$INSTDIR\Uninstall.exe"
    Delete "`$SMPROGRAMS\MixOSC\MixOSC.lnk"
    Delete "`$SMPROGRAMS\MixOSC\Uninstall.lnk"
    RMDir "`$SMPROGRAMS\MixOSC"
    Delete "`$DESKTOP\MixOSC.lnk"
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\MixOSC"
    DeleteRegKey HKCU "Software\MixOSC"
    RMDir "`$INSTDIR"
SectionEnd
"@

Set-Content -Path "$nsiTemp\installer.nsi" -Value $installerNsi
Copy-Item "$PSScriptRoot\..\LICENSE" "$nsiTemp\LICENSE" -Force -ErrorAction SilentlyContinue

Push-Location $nsiTemp
& $nsisPath "$nsiTemp\installer.nsi"
Pop-Location

$distDir = Join-Path (Split-Path $PSScriptRoot -Parent) "dist"
New-Item -ItemType Directory -Force $distDir | Out-Null
$outFile = "mixosc-$pkgVersion.exe"
Copy-Item "$nsiTemp\mixosc-setup.exe" "$distDir\$outFile" -Force -ErrorAction SilentlyContinue

if (Test-Path "$distDir\$outFile") {
    Write-Host "Done: $(Resolve-Path "$distDir\$outFile")"
} else {
    Write-Error "Installer build failed. $outFile was not created."
}
