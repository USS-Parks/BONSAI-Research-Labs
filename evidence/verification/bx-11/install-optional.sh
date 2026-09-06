#!/bin/sh
set -eu
export PATH="$HOME/.local/bin:$PATH"
export UV_PROJECT_ENVIRONMENT=target/linux-venv
uv sync --frozen --extra gymnasium
