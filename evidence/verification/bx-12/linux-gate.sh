#!/bin/sh
set -eu
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export CARGO_BUILD_JOBS=4
uname -srmo
cargo --version
cargo build --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-governor --example feature_admission
cargo build --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-contracts --example feature_lineage
export BONSAI_FEATURE_GOVERNOR="$PWD/target/x86_64-unknown-linux-gnu/debug/examples/feature_admission"
cargo fmt --all --check
cargo clippy --locked --offline --target x86_64-unknown-linux-gnu --workspace --all-targets --all-features -- -D warnings
cargo test --locked --offline --target x86_64-unknown-linux-gnu --workspace --all-features -- --nocapture
cargo run --locked --offline --target x86_64-unknown-linux-gnu -p bonsai-xtask -- schema-check

export UV_PROJECT_ENVIRONMENT=target/linux-venv
uv run --frozen --offline --extra gymnasium pytest --basetemp target/bx12-resumed-linux-pytest-1788713800 -o cache_dir=target/bx12-resumed-linux-pytest-1788713800-cache
uv run --frozen --offline --extra gymnasium python scripts/check_python_protocol.py
