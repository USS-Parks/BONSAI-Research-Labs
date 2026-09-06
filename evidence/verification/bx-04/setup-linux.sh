#!/bin/sh
set -eu
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"
export UV_PROJECT_ENVIRONMENT=target/linux-venv
uv sync --frozen --python /usr/bin/python3
