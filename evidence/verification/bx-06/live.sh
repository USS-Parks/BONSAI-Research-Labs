#!/bin/sh
set -eu
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_BUILD_JOBS=4
cargo build --offline --target x86_64-unknown-linux-gnu -p bonsai-xtask
export XDG_RUNTIME_DIR="/run/user/$(id -u)"
systemd-run --user --wait --pipe --collect --property=Delegate=yes --property=DelegateSubgroup=supervisor --property=RuntimeMaxSec=240 --unit=bonsai-bx06-live --working-directory="$PWD" /usr/bin/python3 evidence/verification/bx-06/live.py "$@"
