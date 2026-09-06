@echo on
uv run --frozen ruff check .
if errorlevel 1 exit /b 1
uv run --frozen pyright
if errorlevel 1 exit /b 1
uv run --frozen pytest --basetemp target/bx05-pytest-final2 -o cache_dir=target/bx05-pytest-cache-final2
if errorlevel 1 exit /b 1
