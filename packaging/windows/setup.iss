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
  Function to prompt user for the EXE file location
  Returns the path to liquidlauncher.exe
*}
function GetExePath(Param: string): string;
var
  defaultPath: string;
  resultPath: string;
begin
  { Try default cross-compiled path }
  defaultPath := ExpandConstant('{src}\..\..\src-tauri\target\x86_64-pc-windows-gnu\release\' + Param);
  if FileExists(defaultPath) then
  begin
    resultPath := defaultPath;
  end
  else
  begin
    { Try alternative path }
    defaultPath := ExpandConstant('{src}\..\..\src-tauri\target\release\' + Param);
    if FileExists(defaultPath) then
    begin
      resultPath := defaultPath;
    end
    else
    begin
      { Prompt user to browse for the file }
      if InputQuery('EXE File Location', 'Please enter the full path to ' + Param + ':', defaultPath, resultPath, 512) then
      begin
        if not FileExists(resultPath) then
        begin
          MsgBox('File not found: ' + resultPath, mbError, MB_OK);
          resultPath := '';
        end;
      end;
    end;
  end;
  Result := resultPath;
end;

{*
  Function to check if the EXE file exists at the specified path
  Used in the [Files] section Check parameter
*}
function CheckExePath(Param: string): boolean;
var
  path: string;
begin
  path := GetExePath(Param);
  Result := (path <> '');
end;

[Files]
; The compiled Windows binary - will prompt user if not found
Source: "..\..\src-tauri\target\x86_64-pc-windows-gnu\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion; Check: CheckExePath('{#MyAppExeName}')

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
