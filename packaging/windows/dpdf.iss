; dpdf Inno Setup Script
; Generates a dual-scope Windows Installer (User / Machine) with UAC elevation options

#define MyAppName "dpdf"
#ifndef MyAppVersion
  #define MyAppVersion "2.0.0"
#endif
#ifndef BinDir
  #define BinDir "..\..\target\windows-package\stage"
#endif
#define MyAppPublisher "Divya"
#define MyAppExeName "dpdf.exe"

[Setup]
AppId={{D9DF821A-5471-48C1-80A7-0E05B5C58A91}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\{#MyAppName}
UninstallDisplayIcon={app}\{#MyAppExeName}
OutputDir=..\..\dist\release
OutputBaseFilename=dpdf-{#MyAppVersion}-setup
SetupIconFile=..\..\assets\app-icon\dpdf.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesAssociations=yes

; Dual-Scope Installation:
; 'lowest' means default to current user without admin rights.
; 'dialog' gives the user the choice between:
;  - "Install for me only" (no admin rights required, %LOCALAPPDATA%\Programs\dpdf)
;  - "Install for all users" (requests admin UAC approval, Program Files\dpdf)
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "contextmenu"; Description: "Add 'Edit with dpdf' to PDF right-click menu"; GroupDescription: "Explorer Integration:"

[Files]
Source: "{#BinDir}\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#BinDir}\dpdf-cli.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#BinDir}\NOTICE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#BinDir}\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#BinDir}\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion
; OCR Models if present
Source: "..\..\target\windows-package\models\*"; DestDir: "{app}\models"; Flags: ignoreversion recursesubdirs createallsubdirs skipifsourcedoesntexist

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; PDF File Association & Context Menu (User or System depending on chosen scope)
Root: HKA; Subkey: "Software\Classes\.pdf\OpenWithProgids"; ValueType: string; ValueName: "dpdf.Document"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\dpdf.Document"; ValueType: string; ValueData: "PDF Document"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\dpdf.Document\DefaultIcon"; ValueType: string; ValueData: """{app}\{#MyAppExeName}"",-2"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\dpdf.Document\shell\open\command"; ValueType: string; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Flags: uninsdeletekey

; Context Menu entry
Root: HKA; Subkey: "Software\Classes\SystemFileAssociations\.pdf\shell\dpdf.Edit"; ValueType: string; ValueData: "Edit with dpdf"; Tasks: contextmenu; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\SystemFileAssociations\.pdf\shell\dpdf.Edit"; ValueType: string; ValueName: "Icon"; ValueData: """{app}\{#MyAppExeName}"",0"; Tasks: contextmenu
Root: HKA; Subkey: "Software\Classes\SystemFileAssociations\.pdf\shell\dpdf.Edit\command"; ValueType: string; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: contextmenu

[Run]
Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Filename: "{app}\{#MyAppExeName}"; Flags: nowait postinstall skipifsilent
