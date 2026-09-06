"""Run the BX-13 two-child protocol tests and retain hosted-CI evidence."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BATCH = ROOT / "target" / f"bx13-ci-{time.time_ns()}"
BATCH.mkdir(parents=True)


def git(*arguments: str, binary: bool = False) -> bytes | str:
    result = subprocess.check_output(["git", *arguments], cwd=ROOT, timeout=120)
    return result if binary else result.decode().strip()


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


command = [
    sys.executable,
    "-B",
    "-m",
    "pytest",
    "python/bonsai-reference/tests/test_option_adapter.py",
    "--basetemp",
    str(BATCH / "pytest-tmp"),
    "-o",
    f"cache_dir={BATCH / 'pytest-cache'}",
    f"--junitxml={BATCH / 'pytest.xml'}",
    "-ra",
]
started = time.monotonic_ns()
try:
    completed = subprocess.run(command, cwd=ROOT, capture_output=True, check=False, timeout=600)
    stdout, stderr, exit_code = completed.stdout, completed.stderr, completed.returncode
except subprocess.TimeoutExpired as error:
    stdout = error.stdout or b""
    stderr = (error.stderr or b"") + b"\nBX13_CI_PROTOCOL_TIMEOUT\n"
    exit_code = 124
elapsed = time.monotonic_ns() - started
(BATCH / "pytest.stdout.txt").write_bytes(stdout)
(BATCH / "pytest.stderr.txt").write_bytes(stderr)

patch = git("diff", "HEAD", "--binary", binary=True)
assert isinstance(patch, bytes)
revision = git("rev-parse", "HEAD")
tree = git("rev-parse", "HEAD^{tree}")
assert isinstance(revision, str) and isinstance(tree, str)
source_identity = {
    "revision": revision,
    "tree": tree,
    "github_sha": os.getenv("GITHUB_SHA"),
    "tracked_patch_bytes": len(patch),
    "tracked_patch_sha256": digest(patch),
    "workflow_sha256": digest((ROOT / ".github/workflows/ci.yml").read_bytes()),
    "test_sha256": digest((ROOT / "python/bonsai-reference/tests/test_option_adapter.py").read_bytes()),
    "wrapper_sha256": digest(Path(__file__).read_bytes()),
}
(BATCH / "source-identity.json").write_text(json.dumps(source_identity, indent=2) + "\n", encoding="utf-8")

host = {
    "system": platform.system(),
    "release": platform.release(),
    "machine": platform.machine(),
    "python_version": platform.python_version(),
    "python_executable_sha256": digest(Path(sys.executable).read_bytes()),
    "github_actions": os.getenv("GITHUB_ACTIONS") == "true",
    "runner_os": os.getenv("RUNNER_OS"),
    "runner_arch": os.getenv("RUNNER_ARCH"),
    "runner_name": os.getenv("RUNNER_NAME"),
    "matrix_os_family": os.getenv("BONSAI_CI_OS_FAMILY"),
    "matrix_arch": os.getenv("BONSAI_CI_ARCH"),
    "matrix_runner": os.getenv("BONSAI_CI_RUNNER"),
    "wsl": "microsoft" in platform.release().lower(),
    "hosted_ci_protocol_evidence": os.getenv("GITHUB_ACTIONS") == "true",
    "os_resource_controller_evidence": False,
    "wsl_governed_matrix_evidence": False,
    "physical_host_acceptance": False,
}
summary = {
    "schema": "bonsai.online-option-ci-protocol/v1",
    "test_file": "python/bonsai-reference/tests/test_option_adapter.py",
    "command": command,
    "exit_code": exit_code,
    "elapsed_ns": elapsed,
    "stdout": {"bytes": len(stdout), "sha256": digest(stdout)},
    "stderr": {"bytes": len(stderr), "sha256": digest(stderr)},
    "source_identity": source_identity,
    "host": host,
}
(BATCH / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"batch": BATCH.relative_to(ROOT).as_posix(), "exit_code": exit_code}), flush=True)
raise SystemExit(exit_code)
