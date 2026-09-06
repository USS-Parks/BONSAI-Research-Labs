#!/bin/sh
set -eu
export PATH="$HOME/.local/bin:$PATH"
export UV_PROJECT_ENVIRONMENT=target/linux-venv
uv run --frozen --offline python scripts/generate_sbom.py
uv run --frozen --offline python scripts/package_rc.py
