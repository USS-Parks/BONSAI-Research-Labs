#!/bin/sh
set -eu
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export CARGO_BUILD_JOBS=4
export UV_PROJECT_ENVIRONMENT=target/linux-venv
cargo build --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-runtime --example adapter_compatibility
uv run --frozen --offline python evidence/verification/bx-09/run.py
