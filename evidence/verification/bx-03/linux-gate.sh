#!/bin/sh
set -eu
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export CARGO_BUILD_JOBS=4
uname -srmo
cargo --version
cargo fmt --all --check
cargo build --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-governor --example linux_authority_probe
cargo clippy --locked --offline --target x86_64-unknown-linux-gnu --workspace --all-targets --all-features -- -D warnings
cargo test --locked --offline --target x86_64-unknown-linux-gnu --workspace --all-features -- --nocapture
cargo run --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-xtask -- schema-check

sh evidence/verification/bx-03/live-gate.sh
