#!/bin/sh
set -eu
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export CARGO_BUILD_JOBS=4
export XDG_RUNTIME_DIR="/run/user/$(id -u)"
cargo build --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-xtask
systemd-run --user --wait --pipe --collect --property=Delegate=yes --property=DelegateSubgroup=supervisor --property=RuntimeMaxSec=180 --unit="bonsai-bx08-interrupt-$(date +%s%N)" --working-directory="$PWD" /usr/bin/python3 evidence/verification/bx-08/interrupt-run.py
