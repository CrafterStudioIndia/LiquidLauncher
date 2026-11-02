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
  Function to get the EXE path - tries multiple locations
  If not found, prompts user with InputQuery
*}
function GetExePath(): string;
var
  path1, path2, path3: string;
begin
  Result := '';
  
  { Try path 1: cross-compiled relative path }
  path1 := '..\..\src-tauri\target\x86_64-pc-windows-gnu\release\liquidlauncher.exe';
  if FileExists(path1) then
  begin
    Result := path1;
    Exit;
  end;
  
  { Try path 2: native build relative path }
  path2 := '..\..\src-tauri\target\release\liquidlauncher.exe';
  if FileExists(path2) then
  begin
    Result := path2;
    Exit;
  end;
  
  { Try path 3: absolute Linux path (for Wine) }
  path3 := '/home/crafter/Documents/Projects/LiquidRemoval/LiquidLauncher/src-tauri/target/x86_64-pc-windows-gnu/release/liquidlauncher.exe';
  if FileExists(path3) then
  begin
    Result := path3;
    Exit;
  end;
  
  { Prompt user if not found at any default location }
  if InputQuery('EXE File Location', 'Enter the full path to liquidlauncher.exe:', '', Result, 1024) then
  begin
    if not FileExists(Result) then
    begin
      MsgBox('File not found: ' + Result, mbError, MB_OK);
      Result := '';
    end;
  end;
end;

{*
  Function to check if EXE exists - called by [Files] Check parameter
*}
function CheckExeExists(Param: string): boolean;
begin
  Result := (GetExePath() <> '');
end;

[Files]
Source: "..\..\src-tauri\target\x86_64-pc-windows-gnu\release\liquidlauncher.exe"; DestDir: "{app}"; Flags: ignoreversion; Check: CheckExeExists('dummy')

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\liquidlauncher.exe"; IconFilename: "{app}\liquidlauncher.exe"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\liquidlauncher.exe"; IconFilename: "{app}\liquidlauncher.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\liquidlauncher.exe"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
