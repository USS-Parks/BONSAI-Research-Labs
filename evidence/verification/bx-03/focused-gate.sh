#!/bin/sh
set -eu
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_BUILD_JOBS=4
cargo fmt --all --check
cargo clippy --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-platform -p bonsai-governor --all-targets --all-features -- -D warnings
cargo build --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-governor --example linux_authority_probe
sh evidence/verification/bx-03/live-gate.sh
