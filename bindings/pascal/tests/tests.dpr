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
  Assert(Length(dNet.Version) >= 5);
end;

procedure TestMACAddress;
begin
  Assert(Length(dNet.MACAddress) = 17);
end;

procedure TestLookupHost;
begin
  Assert(dNet.LookupHost('localhost') = '127.0.0.1');
end;

procedure TestConnectionHealth;
begin
  Assert(not dNet.ConnectionHealth('1.2.3.4', 1234, 10));
  Assert(dNet.ConnectionHealth('54.94.220.237', 443));
end;

procedure TestNtpRequest;
begin
  Assert(dNet.NtpRequest > 0);
end;

begin
  dNet.Load(Concat('../../target/release/', dNet.LIB_NAME));
  TestVersion;
  TestMACAddress;
  TestLookupHost;
  TestConnectionHealth;
  TestNtpRequest;
  Writeln('All tests passed!');
{$IFDEF MSWINDOWS}
  Writeln('Press ENTER to exit ...');
  Readln;
{$ENDIF}
end.
