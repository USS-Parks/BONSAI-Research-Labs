@echo on
wsl -d Ubuntu --exec sh evidence/verification/bx-05/reconcile.sh target/bx05-failures-1788660837367174365/s_profile
if errorlevel 1 exit /b 1
uv run --frozen python evidence/verification/bx-05/reconcile_failures.py target/bx05-failures-1788660837367174365
if errorlevel 1 exit /b 1
wsl -d Ubuntu --exec /usr/bin/python3 evidence/verification/bx-05/preflight_cases.py target/bx05-failures-1788660837367174365/s_profile-manifest.json
if errorlevel 1 exit /b 1
