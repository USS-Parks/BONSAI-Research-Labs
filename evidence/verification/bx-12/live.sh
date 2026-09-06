#!/bin/sh
set -eu
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export UV_PROJECT_ENVIRONMENT=target/linux-venv
export BONSAI_FEATURE_EXAMPLES="$PWD/target/x86_64-unknown-linux-gnu/debug/examples"
uv run --frozen --offline --extra gymnasium python -B evidence/verification/bx-12/generate.py
uv run --frozen --offline --extra gymnasium python -B evidence/verification/bx-12/live.py
