#!/bin/sh
set -eu
export XDG_RUNTIME_DIR="/run/user/$(id -u)"
exec systemd-run --user --wait --pipe --collect --property=Delegate=yes --property=DelegateSubgroup=supervisor --property=RuntimeMaxSec=600 --unit=bonsai-bx13-live --working-directory="$PWD" /usr/bin/python3 evidence/verification/bx-13/live.py "$@"
