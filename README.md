# libduallnet

Rust library which provides network features for Duall ERP.

## C interface

Input strings must be valid UTF-8, NUL-terminated, and backed by readable memory.
Output buffers must be writable and have enough capacity for the full result
including its NUL terminator. Invalid UTF-8 and insufficient capacity return
`-1`. An undersized nonempty buffer receives an empty string; results are never
silently truncated. Export names and Pascal binding signatures are unchanged.
SNTP failures reset the timestamp. The server address is resolved first, so a
DNS failure or an address without port returns `-3`; a read timeout returns `-2`
on every platform (Unix reports it as `EAGAIN`). `dn_connection_health` accepts
bare (`::1`) and bracketed (`[::1]`) IPv6 addresses.

Focused regression tests use only local fixtures:

```powershell
cargo test --target x86_64-pc-windows-msvc regression:: -- --test-threads=1
```

This filter does not run the older tests that require external TCP/NTP servers.

## Linux SONAME and binary packages

Linux builds embed `libduallnet.so.2` as their SONAME. The binary package contains
`libduallnet.so.2.0.3` and links `libduallnet.so.2` and `libduallnet.so`, preserving
both C linker and Pascal loader names. Windows packages contain `duallnet.dll`.

Build and package x64 artifacts (Python 3.11+):

```text
cargo build --release --target x86_64-unknown-linux-gnu
python3 scripts/package_release.py --target x86_64-unknown-linux-gnu
```

On Windows, use `x86_64-pc-windows-msvc` and set
`RUSTFLAGS=-Ctarget-feature=+crt-static` before building. Packages in `dist/`
include bindings, license, the committed Cargo.lock, BUILD.json and SHA256SUMS;
each archive has its own `.sha256`. BUILD.json records source revision, dirty
state, toolchain and, on Linux, SONAME, the minimum required glibc (strong symbol versions only) and the
optional newer glibc versions the binary uses when available (weak, with fallback). A Linux build
from a newer distribution may require newer glibc on the deployment machine.
