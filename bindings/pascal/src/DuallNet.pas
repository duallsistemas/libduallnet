unit DuallNet;

{$IFDEF FPC}
 {$MODE DELPHI}
 {$IFDEF VER3_0}
  {$PUSH}{$MACRO ON}
  {$DEFINE EInvalidOpException := Exception}
  {$POP}
 {$ENDIF}
{$ENDIF}

interface

uses
  SysUtils,
  DateUtils,
  Marshalling,
  libduallnet;

const
  MAC_ADDR_SIZE = 17;
  MAX_IP_SIZE = 45;
  NTP_TIME_SIZE = 24;
  POOL_NTP_ADDR = 'pool.ntp.org';
  GOOGLE_NTP_ADDR = 'time.google.com';
  NTP_PORT = 123;

resourcestring
  SInvalidFunctionArgument = 'Invalid function argument.';
  SUnknownErrorInFunction = 'Unknown error in function: %s.';

type

  { EdNet }

  EdNet = class(EInvalidOpException);

  { TdConnectionStatus }

  TdConnectionStatus = (csOK, csTimeOut, csCannotConnect);

  { dNet }

  dNet = packed record
  public const
    LIB_NAME = libduallnet.DN_LIB_NAME;
  public
    class procedure Load(const ALibraryName: TFileName = LIB_NAME); static;
    class procedure Unload; static;
    class function Version: string; static;
    class function MACAddress: string; static;
    class function LookupHost(const AHostName: string;
      APreferIPv4: Boolean = True): string; static;
    class function ConnectionHealth(const AIP: string; APort: Word;
      ATimeout: UInt64 = 3000): TdConnectionStatus; static;
    class function IsConnectable(const AIP: string; APort: Word;
      ATimeout: UInt64 = 3000): Boolean; static;
    class function NtpRequest(const APool: string = POOL_NTP_ADDR;
      APort: Word = NTP_PORT): TDateTime; static;
  end;

implementation

procedure RaiseInvalidFunctionArgument; inline;
begin
  raise EdNet.Create(SInvalidFunctionArgument);
end;

procedure RaiseUnknownErrorInFunction(const AFuncName: string); inline;
begin
  raise EdNet.CreateFmt(SUnknownErrorInFunction, [AFuncName]);
end;

{ dNet }

class procedure dNet.Load(const ALibraryName: TFileName);
begin
  Unload;
  libduallnet.Load(ALibraryName);
end;

class procedure dNet.Unload;
begin
  libduallnet.Unload;
end;

class function dNet.Version: string;
begin
  libduallnet.Check;
  Result := TMarshal.ToString(libduallnet.dn_version);
end;

class function dNet.MACAddress: string;
var
  A: array[0..MAC_ADDR_SIZE] of cchar;
  R: cint;
begin
  libduallnet.Check;
  R := libduallnet.dn_mac_address(@A[0], SizeOf(A));
  case R of
    -1: RaiseInvalidFunctionArgument;
    -2: raise EdNet.Create('No address found.');
    -3: RaiseUnknownErrorInFunction('dNet.MACAddress');
  end;
  Result := TMarshal.ToString(@A[0]);
end;

class function dNet.LookupHost(const AHostName: string;
  APreferIPv4: Boolean): string;
var
  M: TMarshaller;
  A: array[0..MAX_IP_SIZE] of cchar;
  R: cint;
begin
  libduallnet.Check;
  A[0] := 0;
  R := libduallnet.dn_lookup_host(M.ToCString(AHostName), APreferIPv4,
    @A[0], SizeOf(A));
  case R of
    -1: RaiseInvalidFunctionArgument;
    -2: raise EdNet.Create('No MAC address found.');
    -3: RaiseUnknownErrorInFunction('dNet.LookupHost');
  end;
  Result := TMarshal.ToString(@A[0]);
end;

class function dNet.ConnectionHealth(const AIP: string; APort: Word;
  ATimeout: UInt64): TdConnectionStatus;
var
  M: TMarshaller;
  R: cint;
begin
  libduallnet.Check;
  R := libduallnet.dn_connection_health(M.ToCString(AIP), APort, ATimeout);
  case R of
    -1: RaiseInvalidFunctionArgument;
    -2: Result := csTimeOut;
    -3: Result := csCannotConnect;
    -4: RaiseUnknownErrorInFunction('dNet.ConnectionHealth');
  else
    Result := csOK;
  end;
end;

class function dNet.IsConnectable(const AIP: string; APort: Word;
  ATimeout: UInt64): Boolean;
begin
  Result := dNet.ConnectionHealth(AIP, APort, ATimeout) = csOK;
end;

class function dNet.NtpRequest(const APool: string; APort: Word): TDateTime;
var
  M: TMarshaller;
  R, TS: cint;
begin
  libduallnet.Check;
  R := dn_ntp_request(M.ToCString(APool), APort, @TS);
  case R of
    -1: RaiseInvalidFunctionArgument;
    -2: raise EdNet.Create('NTP error.');
  end;
  Result :=
{$IFDEF FPC}
    UniversalTimeToLocal
{$ELSE}
    TTimeZone.Local.ToLocalTime
{$ENDIF}(UnixToDateTime(TS));
end;

end.
