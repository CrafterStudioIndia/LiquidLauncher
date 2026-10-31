; Inno Setup 6 Script for LiquidLauncher
; Generated for standalone Windows installer packaging

#define MyAppName "LiquidLauncher"
#define MyAppVersion "0.7.1"
#define MyAppPublisher "CrafterStudio"
#define MyAppURL "https://github.com/CrafterStudioIndia/LiquidLauncher"
#define MyAppExeName "liquidlauncher.exe"

[Setup]
AppId={{5E285F5D-5441-4DAE-B4A1-F28CE0BD80D1}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
LicenseFile=..\..\LICENSE
OutputDir=..\..\src-tauri\target\release\bundle\inno
OutputBaseFilename=LiquidLauncher_{#MyAppVersion}_x64_setup
SetupIconFile=..\..\src-tauri\icons\icon.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; The compiled Windows binary from x86_64-pc-windows-gnu or native build
Source: "..\..\src-tauri\target\x86_64-pc-windows-gnu\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion; Check: FileExists(ExpandConstant('{src}\..\..\src-tauri\target\x86_64-pc-windows-gnu\release\{#MyAppExeName}'))
Source: "..\..\src-tauri\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion; Check: not FileExists(ExpandConstant('{src}\..\..\src-tauri\target\x86_64-pc-windows-gnu\release\{#MyAppExeName}'))

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

