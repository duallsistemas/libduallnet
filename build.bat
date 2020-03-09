:::::::::::::::::::::::::::::::::::::::::::
:: Copyright (C) 2019 Duall Sistemas Ltda.
:::::::::::::::::::::::::::::::::::::::::::

set RUSTFLAGS=-Ctarget-feature=+crt-static
rustup target add i686-pc-windows-msvc
cargo build --target=i686-pc-windows-msvc --release
