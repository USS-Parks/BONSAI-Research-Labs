#!/bin/sh
set -eu
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export CARGO_BUILD_JOBS=4
cargo build --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-lineage --example durable_probe
/usr/bin/python3 evidence/verification/bx-08/run.py
