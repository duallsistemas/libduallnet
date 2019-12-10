program tests;

{$IFDEF FPC}
 {$MODE DELPHI}
{$ENDIF}
{$IFDEF MSWINDOWS}
 {$APPTYPE CONSOLE}
{$ENDIF}
{$ASSERTIONS ON}

uses
  SysUtils,
  DuallNet;

procedure TestVersion;
begin
  Assert(Length(TdNet.Version) >= 5);
end;

procedure TestMACAddress;
begin
  Assert(Length(TdNet.MACAddress) = 17);
end;

procedure TestLookupHost;
begin
  Assert(TdNet.LookupHost('localhost') = '127.0.0.1');
end;

procedure TestNtpRequest;
begin
  Assert(TdNet.NtpRequest > 0);
end;

begin
  TdNet.Load(Concat('../../target/release/', TdNet.LIB_NAME));
  TestVersion;
  TestMACAddress;
  TestLookupHost;
  TestNtpRequest;
  Writeln('All tests passed!');
{$IFDEF MSWINDOWS}
  Writeln('Press ENTER to exit ...');
  Readln;
{$ENDIF}
end.
