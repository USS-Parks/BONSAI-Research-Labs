#!/bin/sh
set -eu
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export CARGO_BUILD_JOBS=4
uname -srmo
cargo --version
cargo fmt --all --check
cargo clippy --locked --offline --target x86_64-unknown-linux-gnu --workspace --all-targets --all-features -- -D warnings
cargo test --locked --offline --target x86_64-unknown-linux-gnu --workspace --all-features -- --nocapture
cargo run --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-xtask -- schema-check

export UV_PROJECT_ENVIRONMENT=target/linux-venv
uv run --frozen --offline pytest --basetemp target/bx05-linux-pytest-1788661204833 -o cache_dir=target/bx05-linux-pytest-1788661204833-cache
