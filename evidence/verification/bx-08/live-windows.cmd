@echo on
cargo build --locked --offline -p bonsai-lineage --example durable_probe
if errorlevel 1 exit /b 1
uv run --frozen python evidence/verification/bx-08/run.py
if errorlevel 1 exit /b 1
