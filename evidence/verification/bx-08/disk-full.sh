#!/bin/sh
set -eu
base="target/bx08-diskfull-$(date +%s%N)"
mkdir "$base"
mkdir "$base/mount"
printf 'preserve-user-data\n' > "$base/user-data.txt"
unshare --user --map-root-user --mount --fork sh evidence/verification/bx-08/disk-full-inner.sh "$base"
printf '%s\n' "$base"
