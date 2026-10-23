#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{DB914DBB-D16E-4120-BD4C-FEA6BC09B482}
AppName=trame
AppVersion={#AppVersion}
AppPublisher=Florent Leterme
AppPublisherURL=https://github.com/LetermeFlorent/trame
AppSupportURL=https://github.com/LetermeFlorent/trame/issues
AppUpdatesURL=https://github.com/LetermeFlorent/trame/releases
DefaultDirName={autopf}\trame
DefaultGroupName=trame
DisableProgramGroupPage=yes
DisableDirPage=auto
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog commandline
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SourceDir=..\..
OutputDir=target\installer
OutputBaseFilename=trame-{#AppVersion}-setup
SetupIconFile=assets\trame.ico
UninstallDisplayIcon={app}\trame.exe
UninstallDisplayName=trame
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "target\release\trame.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion

[InstallDelete]
Type: files; Name: "{app}\trame.exe.old"
Type: files; Name: "{app}\trame.exe.new"

[UninstallDelete]
Type: files; Name: "{app}\trame.exe.old"
Type: files; Name: "{app}\trame.exe.new"

[Icons]
Name: "{autoprograms}\trame"; Filename: "{app}\trame.exe"
Name: "{autodesktop}\trame"; Filename: "{app}\trame.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\trame.exe"; Description: "{cm:LaunchProgram,trame}"; Flags: nowait postinstall skipifsilent
Filename: "{app}\trame.exe"; Parameters: "--updated"; Flags: nowait runasoriginaluser; Check: Relaunching

[Code]
function Relaunching: Boolean;
begin
  Result := ExpandConstant('{param:relaunch|no}') = 'yes';
end;
