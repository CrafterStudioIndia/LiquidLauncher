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

[Code]
{*
  Function to check if the EXE exists at expected locations
  Shows error message with instructions if not found
*}
function CheckExeExists(Param: string): boolean;
var
  msg: string;
begin
  { Try all possible locations }
  if FileExists('..\..\src-tauri\target\x86_64-pc-windows-gnu\release\' + Param) or
     FileExists('..\..\src-tauri\target\release\' + Param) or
     FileExists('/home/crafter/Documents/Projects/LiquidRemoval/LiquidLauncher/src-tauri/target/x86_64-pc-windows-gnu/release/' + Param) then
  begin
    Result := true;
    Exit;
  end;
  
  { File not found - show error with instructions }
  msg := 'ERROR: ' + Param + ' not found!' + #13#10 + #13#10 +
         'Please make sure the Windows executable has been built.' + #13#10 +
         'Expected location:' + #13#10 +
         '  ..\..\src-tauri\target\x86_64-pc-windows-gnu\release\' + Param + #13#10 + #13#10 +
         'To build the EXE, run:' + #13#10 +
         '  tauri build --target x86_64-pc-windows-gnu --no-bundle';
  MsgBox(msg, mbError, MB_OK);
  Result := false;
end;

[Files]
; The compiled Windows binary - checks multiple locations
Source: "..\..\src-tauri\target\x86_64-pc-windows-gnu\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion; Check: CheckExeExists('{#MyAppExeName}')

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
