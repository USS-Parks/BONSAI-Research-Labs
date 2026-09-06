#!/bin/sh
set -eu
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export CARGO_BUILD_JOBS=4
export UV_PROJECT_ENVIRONMENT=target/linux-venv
uv sync --frozen --extra gymnasium
cargo build --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-xtask
export XDG_RUNTIME_DIR="/run/user/$(id -u)"
systemd-run --user --wait --pipe --collect --property=Delegate=yes --property=DelegateSubgroup=supervisor --property=RuntimeMaxSec=600 --unit=bonsai-bx11-live --working-directory="$PWD" /usr/bin/python3 evidence/verification/bx-11/live.py
