#!/bin/sh
set -eu
[ "$#" -eq 1 ] || { echo 'usage: negative-corpus.sh OBSERVER_ROOT' >&2; exit 2; }
run_root=$1
set -- "$HOME"/.cargo/registry/src/*/protoc-bin-vendored-linux-x86_64-3.2.0/bin/protoc
[ "$#" -eq 1 ] && [ -x "$1" ] || { echo 'locked protoc 3.2.0 cache entry required' >&2; exit 2; }
mkdir -p target/bx06-audit-proto
"$1" --version
"$1" -I proto --python_out=target/bx06-audit-proto proto/bonsai/event/v1/envelope.proto
target/linux-venv/bin/python evidence/verification/bx-06/negative_corpus.py "$run_root"
