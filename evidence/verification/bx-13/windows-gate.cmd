@echo on
set "BX13_TEST_ROOT=target/bx13-full-windows-%RANDOM%-%RANDOM%"
if exist "%BX13_TEST_ROOT%" exit /b 1
if exist "%BX13_TEST_ROOT%-cache" exit /b 1
cargo build --locked --offline -p bonsai-governor --example feature_admission
if errorlevel 1 exit /b 1
cargo build --locked --offline -p bonsai-contracts --example feature_lineage
if errorlevel 1 exit /b 1
cargo fmt --all --check
if errorlevel 1 exit /b 1
cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings
if errorlevel 1 exit /b 1
cargo test --locked --offline --workspace --all-features
if errorlevel 1 exit /b 1
cargo xtask schema-check
if errorlevel 1 exit /b 1
uv run --frozen ruff check .
if errorlevel 1 exit /b 1
uv run --frozen pyright
if errorlevel 1 exit /b 1
uv run --frozen pytest --basetemp "%BX13_TEST_ROOT%" -o "cache_dir=%BX13_TEST_ROOT%-cache"
if errorlevel 1 exit /b 1
uv run --frozen python -B scripts/check_docs.py
if errorlevel 1 exit /b 1
uv run --frozen python -B scripts/check_adrs.py
if errorlevel 1 exit /b 1
uv run --frozen python -B scripts/check_license.py
if errorlevel 1 exit /b 1
uv run --frozen python -B scripts/check_governance_ledgers.py
if errorlevel 1 exit /b 1
uv run --frozen python -B scripts/check_terminology.py
if errorlevel 1 exit /b 1
uv run --frozen python -B scripts/check_ci.py
if errorlevel 1 exit /b 1
uv run --frozen python -B scripts/check_m4.py
if errorlevel 1 exit /b 1
uv run --frozen python scripts/check_python_protocol.py
if errorlevel 1 exit /b 1
