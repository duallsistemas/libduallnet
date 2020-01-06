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
  Assert(dNet.ConnectionHealth('127.0.0.1', 1, 0) = csCannotConnect);
  Assert(dNet.ConnectionHealth('1.2.3.4', 123, 10) = csTimeOut);
  Assert(dNet.ConnectionHealth('54.94.220.237', 443) = csOK);
end;

procedure TestIsConnectable;
begin
  Assert(not dNet.IsConnectable('127.0.0.1', 1, 10));
  Assert(not dNet.IsConnectable('1.2.3.4', 123, 10));
  Assert(dNet.IsConnectable('54.94.220.237', 443));
end;

procedure TestSntpRequest;
var
  T1, T2: TDateTime;
begin
  T1 := dNet.SntpRequest;
  Sleep(2000);
  T2 := dNet.SntpRequest;
  Assert(T2 > T1);
end;

begin
{$IFDEF MSWINDOWS}
  dNet.Load(Concat('../../target/i686-pc-windows-msvc/release/', dNet.LIB_NAME));
{$ELSE}
  dNet.Load(Concat('../../target/release/', dNet.LIB_NAME));
{$ENDIF}
  TestVersion;
  TestMACAddress;
  TestLookupHost;
  TestConnectionHealth;
  TestIsConnectable;
  TestSntpRequest;
  Writeln('All tests passed!');
{$IFDEF MSWINDOWS}
  Writeln('Press ENTER to exit ...');
  Readln;
{$ENDIF}
end.
