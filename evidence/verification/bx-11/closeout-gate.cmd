@echo on
uv run --frozen ruff check .
if errorlevel 1 exit /b 1
uv run --frozen python -B scripts/check_docs.py
if errorlevel 1 exit /b 1
uv run --frozen python -B scripts/check_governance_ledgers.py
if errorlevel 1 exit /b 1
uv run --frozen python -B scripts/check_terminology.py
if errorlevel 1 exit /b 1
uv run --frozen python -B scripts/check_m4.py
if errorlevel 1 exit /b 1
