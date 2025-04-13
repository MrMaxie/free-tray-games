;#define MyAppName GetEnv("MyAppName")
;#define MyAppVersion GetEnv("MyAppVersion")
#define MyAppPublisher "Maxiej ""Maxie"" Mieńko"
#define MyAppURL "https://github.com/MrMaxie/free-tray-games"
;#define MyAppExeName GetEnv("MyAppExeName")

[Setup]
AppId={{79CC127B-B677-4ECC-B922-60B3CFB157C7}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
UninstallDisplayIcon={app}\{#MyAppExeName}
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=.\LICENSE.rtf
PrivilegesRequired=lowest
OutputBaseFilename={#OutputBaseFilename}
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: ".\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "FreeTrayGames"; ValueData: """{app}\freetraygames.exe"""; Flags: uninsdeletevalue

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; AppUserModelID: "FreeTrayGames.App"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "taskkill.exe"; \
  Parameters: "/IM freetraygames.exe /F"; \
  Flags: runhidden runascurrentuser skipifdoesntexist

[UninstallDelete]
Name: "{app}"; Type: filesandordirs
