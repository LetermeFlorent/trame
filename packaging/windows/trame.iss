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
Name: "contextmenu"; Description: "{cm:ContextMenu}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "target\release\trame.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion
Source: "assets\lucide\LICENSE"; DestDir: "{app}"; DestName: "LICENSE-lucide.txt"; Flags: ignoreversion

[InstallDelete]
Type: files; Name: "{app}\trame.exe.old"
Type: files; Name: "{app}\trame.exe.new"

[UninstallDelete]
Type: files; Name: "{app}\trame.exe.old"
Type: files; Name: "{app}\trame.exe.new"

[Registry]
Root: HKCU; Subkey: "Software\Classes\Directory\shell\trame"; Flags: uninsdeletekey dontcreatekey
Root: HKCU; Subkey: "Software\Classes\Directory\Background\shell\trame"; Flags: uninsdeletekey dontcreatekey
Root: HKCU; Subkey: "Software\Classes\Directory\shell\trame"; ValueType: string; ValueData: "{cm:OpenInTrame}"; Tasks: contextmenu; Check: not Relaunching
Root: HKCU; Subkey: "Software\Classes\Directory\shell\trame"; ValueType: string; ValueName: "Icon"; ValueData: """{app}\trame.exe"",0"; Tasks: contextmenu; Check: not Relaunching
Root: HKCU; Subkey: "Software\Classes\Directory\shell\trame\command"; ValueType: string; ValueData: """{app}\trame.exe"" ""%V"""; Tasks: contextmenu; Check: not Relaunching
Root: HKCU; Subkey: "Software\Classes\Directory\Background\shell\trame"; ValueType: string; ValueData: "{cm:OpenInTrame}"; Tasks: contextmenu; Check: not Relaunching
Root: HKCU; Subkey: "Software\Classes\Directory\Background\shell\trame"; ValueType: string; ValueName: "Icon"; ValueData: """{app}\trame.exe"",0"; Tasks: contextmenu; Check: not Relaunching
Root: HKCU; Subkey: "Software\Classes\Directory\Background\shell\trame\command"; ValueType: string; ValueData: """{app}\trame.exe"" ""%V"""; Tasks: contextmenu; Check: not Relaunching

[CustomMessages]
english.ContextMenu=Add "Open in trame" to the right-click menu of folders
french.ContextMenu=Ajouter "Ouvrir dans trame" au menu clic droit des dossiers
english.OpenInTrame=Open in trame
french.OpenInTrame=Ouvrir dans trame

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
