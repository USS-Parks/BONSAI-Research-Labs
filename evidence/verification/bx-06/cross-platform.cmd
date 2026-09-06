@echo on
target\debug\bonsai-xtask.exe verify-run --root target/bx06-live-1788664706206224389/run/observer --receipt-sha256 c8856439a0ace52a3c29307f67dee0b4ae5902cdb1e3ce50b2a3ec398ab15a32
if errorlevel 1 exit /b 1
uv run --frozen python evidence/verification/bx-06/negative_corpus.py target/bx06-live-1788664647209476649/run/observer
if errorlevel 1 exit /b 1
