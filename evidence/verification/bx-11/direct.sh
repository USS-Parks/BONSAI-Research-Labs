#!/bin/sh
set -eu
export PATH="$HOME/.local/bin:$PATH"
export UV_PROJECT_ENVIRONMENT=target/linux-venv
uv run --frozen --extra gymnasium pytest python/bonsai-reference/tests/test_gymnasium_adapter.py --basetemp target/bx11-direct-final-1788679750 -o cache_dir=target/bx11-direct-cache-final-1788679750
