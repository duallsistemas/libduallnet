unit libduallnet;

{$IFDEF FPC}
 {$MODE DELPHI}
 {$PACKRECORDS C}
 {$IFDEF VER3_0}
  {$PUSH}{$MACRO ON}
  {$DEFINE MarshaledAString := PAnsiChar}
  {$DEFINE EInvalidOpException := Exception}
  {$IFDEF VER3_0_0}
   {$DEFINE EFileNotFoundException := Exception}
  {$ENDIF}
  {$POP}
 {$ENDIF}
{$ENDIF}

interface

uses
  SysUtils,
  StrUtils,
{$IFDEF MSWINDOWS}
  Windows,
{$ENDIF}
{$IFDEF FPC}
  DynLibs,
{$ENDIF}
  SyncObjs;


const
  SharedPrefix = {$IFDEF MSWINDOWS}''{$ELSE}'lib'{$ENDIF};
{$IF (NOT DEFINED(FPC)) OR DEFINED(VER3_0)}
  SharedSuffix =
 {$IF DEFINED(MSWINDOWS)}
    'dll'
 {$ELSEIF DEFINED(MACOS)}
    'dylib'
 {$ELSE}
    'so'
 {$ENDIF};
{$ENDIF}
  DN_LIB_NAME = Concat(SharedPrefix, 'duallnet.', SharedSuffix);

{$IFDEF FPC}
 {$IFDEF VER3_0}
const
  NilHandle = DynLibs.NilHandle;
type
  TLibHandle = DynLibs.TLibHandle;
 {$ENDIF}
{$ELSE}
const
  NilHandle = HMODULE(0);
type
  TLibHandle = HMODULE;
{$ENDIF}

resourcestring
  SdnLibEmptyName = 'Empty library name.';
  SdnLibNotLoaded = 'Library ''%s'' not loaded.';
  SdnLibInvalid = 'Invalid library ''%s''.';

type
  Pcvoid = Pointer;
  Pcchar = MarshaledAString;
  cchar = Byte;
  cint = Int32;
  Pcint = ^Int32;
  csize_t = NativeUInt;

  EdnLibNotLoaded = class(EFileNotFoundException);

var
  dn_version: function: Pcchar; cdecl;

  dn_mac_address: function(mac_addr: Pcchar; size: csize_t): cint; cdecl;

  dn_lookup_host: function(const hostname: Pcchar; prefer_ipv4: Boolean;
    ip: Pcchar; size: csize_t): cint; cdecl;

  dn_ntp_request: function(const pool: Pcchar; port: cint;
    timestamp: Pcint): cint; cdecl;

function TryLoad(const ALibraryName: TFileName): Boolean;

procedure Load(const ALibraryName: TFileName);

procedure Unload;

procedure Check;

implementation

var
  GCS: TCriticalSection;
  GLibHandle: TLibHandle = NilHandle;
  GLibLastName: TFileName = '';

function TryLoad(const ALibraryName: TFileName): Boolean;
begin
  if ALibraryName = '' then
    raise EArgumentException.Create(SdnLibEmptyName);
  GCS.Acquire;
  try
    if GLibHandle <> NilHandle then
      FreeLibrary(GLibHandle);
    GLibHandle := SafeLoadLibrary(ALibraryName);
    if GLibHandle = NilHandle then
      Exit(False);
    GLibLastName := ALibraryName;
    dn_version := GetProcAddress(GLibHandle, 'dn_version');
    dn_mac_address := GetProcAddress(GLibHandle, 'dn_mac_address');
    dn_lookup_host := GetProcAddress(GLibHandle, 'dn_lookup_host');
    dn_ntp_request := GetProcAddress(GLibHandle, 'dn_ntp_request');
    Result := True;
  finally
    GCS.Release;
  end;
end;

procedure Load(const ALibraryName: TFileName);
begin
  if not TryLoad(ALibraryName) then
  begin
{$IFDEF MSWINDOWS}
    if GetLastError = ERROR_BAD_EXE_FORMAT then
      raise EdnLibNotLoaded.CreateFmt(SdnLibInvalid, [ALibraryName]);
{$ENDIF}
    raise EdnLibNotLoaded.CreateFmt(SdnLibNotLoaded, [ALibraryName])
  end;
end;

procedure Unload;
begin
  GCS.Acquire;
  try
    if (GLibHandle = NilHandle) or (not FreeLibrary(GLibHandle)) then
      Exit;
    GLibHandle := NilHandle;
    GLibLastName := '';
    dn_version := nil;
    dn_mac_address := nil;
    dn_lookup_host := nil;
    dn_ntp_request := nil;
  finally
    GCS.Release;
  end;
end;

procedure Check;
begin
  if GLibHandle = NilHandle then
    raise EdnLibNotLoaded.CreateFmt(SdnLibNotLoaded,
      [IfThen(GLibLastName = '', DN_LIB_NAME, GLibLastName)]);
end;

initialization
  GCS := TCriticalSection.Create;
  TryLoad(DN_LIB_NAME);

finalization
  Unload;
  FreeAndNil(GCS);

end.
