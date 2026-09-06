"""Retain native live compatibility evidence and exact source identities."""
from __future__ import annotations

import hashlib
import json
import platform
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
def sources() -> dict[str, str]:
    paths = [path for folder in ["crates", "proto", "python/bonsai-reference/src", "fixtures/adapter-compatibility/v1"]
             for path in (ROOT / folder).rglob("*")
             if path.is_file() and path.suffix in {".rs", ".py", ".pyi", ".proto", ".toml", ".json", ".sql"}]
    paths += [ROOT / name for name in ["Cargo.toml", "Cargo.lock", "pyproject.toml", "uv.lock"]]
    return {path.relative_to(ROOT).as_posix(): digest(path) for path in sorted(paths)}
def main() -> None:
    before = sources()
    output = ROOT / "target" / f"bx09-live-{time.time_ns()}"
    executable = ROOT / ("target/debug/examples/adapter_compatibility.exe" if sys.platform == "win32"
                         else "target/x86_64-unknown-linux-gnu/debug/examples/adapter_compatibility")
    result = subprocess.run([str(executable), sys.executable, str(output)], cwd=ROOT,
                            capture_output=True, text=True, timeout=120)
    (output / "probe.stdout.txt").write_text(result.stdout, encoding="utf-8")
    (output / "probe.stderr.txt").write_text(result.stderr, encoding="utf-8")
    if result.returncode:
        raise RuntimeError(f"probe failed: {output}: {result.stderr}")
    assert sources() == before, "SOURCE_CHANGED_DURING_PROBE"
    reports = json.loads((output / "reports.json").read_text())
    assert len(reports) == 12
    assert all(report["verdict"] == "certified" and len(report["checks"]) == 7 for report in reports)
    summary = {"schema": "bonsai.adapter-compatibility-live/v1", "host": platform.platform(),
               "python": sys.executable, "output": output.relative_to(ROOT).as_posix(),
               "executable_sha256": digest(executable), "sources": before,
               "result": json.loads(result.stdout), "reports_sha256": digest(output / "reports.json")}
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: value for key, value in summary.items() if key != "sources"}, indent=2))
if __name__ == "__main__":
    main()
