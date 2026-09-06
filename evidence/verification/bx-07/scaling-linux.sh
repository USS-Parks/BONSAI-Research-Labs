#!/bin/sh
set -eu
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_BUILD_JOBS=4
cargo build --offline --target x86_64-unknown-linux-gnu -p bonsai-lineage --example incremental_scaling
/usr/bin/python3 evidence/verification/bx-07/scaling.py
