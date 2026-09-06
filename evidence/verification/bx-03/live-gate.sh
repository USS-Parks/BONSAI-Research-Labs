#!/bin/sh
set -eu
uname -srmo
probe=target/x86_64-unknown-linux-gnu/debug/examples/linux_authority_probe
"$probe" denied
export XDG_RUNTIME_DIR="/run/user/$(id -u)"
systemd-run --user --wait --pipe --collect --property=Delegate=yes --property=DelegateSubgroup=supervisor --property=RuntimeMaxSec=50 --unit=bonsai-bx03-live --working-directory="$PWD" "$PWD/$probe"
