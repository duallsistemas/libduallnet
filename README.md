# libduallnet

Rust library which provides network features for Duall ERP.

## C interface

Input strings must be valid UTF-8, NUL-terminated, and backed by readable memory.
Output buffers must be writable and have enough capacity for the full result
including its NUL terminator. Invalid UTF-8 and insufficient capacity return
`-1`. An undersized nonempty buffer receives an empty string; results are never
silently truncated. Export names and Pascal binding signatures are unchanged.
SNTP failures reset the timestamp and are classified using the returned I/O error.

Focused regression tests use only local fixtures:

```powershell
cargo test --target x86_64-pc-windows-msvc regression:: -- --test-threads=1
```

This filter does not run the older tests that require external TCP/NTP servers.
