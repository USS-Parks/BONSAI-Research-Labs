@echo on
cargo build --locked --offline -p bonsai-runtime --example adapter_compatibility
if errorlevel 1 exit /b 1
uv run --frozen python evidence/verification/bx-09/run.py
if errorlevel 1 exit /b 1
