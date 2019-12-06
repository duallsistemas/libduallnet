#!/bin/sh
###############################################################
# Copyright (C) 2019 Duall Sistemas Ltda.
###############################################################

set -e

RUSTFLAGS="-C link-arg=-s" cargo build --release
