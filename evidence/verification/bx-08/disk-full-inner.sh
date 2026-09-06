#!/bin/sh
set -eu
base="$1"
mount -t tmpfs -o size=2M tmpfs "$base/mount"
set +e
target/x86_64-unknown-linux-gnu/debug/examples/durable_probe grow "$base/mount/run" 100000 > "$base/grow.stdout.txt" 2> "$base/grow.stderr.txt"
status=$?
set -e
printf '%s\n' "$status" > "$base/exit-code.txt"
test "$status" -eq 1
df -B1 "$base/mount" > "$base/filesystem.txt"
cp -a "$base/mount/run" "$base/retained"
target/x86_64-unknown-linux-gnu/debug/examples/durable_probe recover "$base/retained" 0 > "$base/recover.stdout.txt" 2> "$base/recover.stderr.txt"
/usr/bin/python3 evidence/verification/bx-08/check-disk-full.py "$base"
