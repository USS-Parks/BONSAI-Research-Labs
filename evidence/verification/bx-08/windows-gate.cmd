@echo on
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
uv run --frozen pytest --basetemp target/bx08-pytest-1788669414872-final -o cache_dir=target/bx08-pytest-1788669414872-cache
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
