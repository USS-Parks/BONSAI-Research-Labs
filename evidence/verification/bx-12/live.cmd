@echo on
uv run --frozen python -B evidence/verification/bx-12/generate.py
if errorlevel 1 exit /b 1
uv run --frozen python -B evidence/verification/bx-12/live.py
if errorlevel 1 exit /b 1
