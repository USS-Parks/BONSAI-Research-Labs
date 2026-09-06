#!/bin/sh
set -eu
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export CARGO_BUILD_JOBS=4
export UV_PROJECT_ENVIRONMENT=target/linux-venv
uv sync --frozen --extra gymnasium
cargo build --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-xtask
uv run --frozen python -B evidence/verification/bx-13/generate.py
