:::::::::::::::::::::::::::::::::::::::::::
:: Copyright (C) 2021 Duall Sistemas Ltda.
:::::::::::::::::::::::::::::::::::::::::::

set RUSTFLAGS=-Ctarget-feature=+crt-static
cargo build --release
