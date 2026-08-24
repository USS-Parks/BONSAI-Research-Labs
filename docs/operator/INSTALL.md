# Install BONSAI

BONSAI is a local measurement instrument. These steps install the source workspace. They do not publish packages and do not claim instrument completion.

## Prerequisites

- Rust 1.96.0 (see `rust-toolchain.toml`)
- Python 3.12–3.14
- uv 0.11.29
- Git

Clone the public repository and enter the root:

```text
git clone https://github.com/USS-Parks/BONSAI-Research-Labs.git
cd BONSAI-Research-Labs
```

## Offline-friendly restore

After the first fetch of locked dependencies:

```text
uv sync --frozen
cargo test --offline --workspace --all-features
uv run --frozen python -B scripts/offline_restore.py
```

`scripts/offline_restore.py --vendor` writes `vendor/` from `Cargo.lock` when you need a transportable tree. Hosted CI offline success is not a physical clean-machine attestation.

## Verify the workspace

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo xtask schema-check
uv run --frozen ruff check .
uv run --frozen pyright
uv run --frozen pytest
```

Passing these gates is not physical-host, energy, long-duration, or evaluated-agent evidence.
