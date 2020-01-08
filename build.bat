:::::::::::::::::::::::::::::::::::::::::::
:: Copyright (C) 2019 Duall Sistemas Ltda.
:::::::::::::::::::::::::::::::::::::::::::

set RUSTFLAGS=-Ctarget-feature=+crt-static
cargo build --target=i686-pc-windows-msvc --release
