"""Reverify all archived BX-11 runs against the current additive contracts."""
import hashlib
import json
import subprocess
import time
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BINARY = ROOT / "target/x86_64-unknown-linux-gnu/debug/bonsai-xtask"
ARCHIVE = ROOT / "evidence/verification/bx-11/external-environment.zip"
OUTPUT = ROOT / "target" / f"bx13-historical-{time.time_ns()}"
OUTPUT.mkdir()
results = []
with zipfile.ZipFile(ARCHIVE) as archive:
    inventory = json.loads(archive.read("manifest.json"))
    names = [name for name in archive.namelist() if name.startswith("target/") and name.endswith("/summary.json")]
    assert len(names) == 1
    summary = json.loads(archive.read(names[0]))
    assert len(summary["runs"]) == 4
    for index, row in enumerate(summary["runs"]):
        prefix = row["case"] + "/run/observer/"
        members = [name for name in archive.namelist() if name.startswith(prefix)]
        assert members
        for name in members:
            path = (ROOT / name).resolve()
            assert path.is_relative_to(ROOT / "target") and path.is_file()
            raw = path.read_bytes()
            assert len(raw) == inventory[name]["bytes"]
            assert hashlib.sha256(raw).hexdigest() == inventory[name]["sha256"]
            assert raw == archive.read(name)
        command = [str(BINARY), "verify-run", "--root", str(ROOT / prefix),
                   "--receipt-sha256", row["receipt_sha256"]]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=60)
        (OUTPUT / f"run-{index}.stdout.txt").write_bytes(result.stdout)
        (OUTPUT / f"run-{index}.stderr.txt").write_bytes(result.stderr)
        assert result.returncode == 0, result.stderr.decode()
        results.append({"case": row["case"], "receipt_sha256": row["receipt_sha256"],
                        "verified_archive_members": len(members), "exit_code": result.returncode})
result = {"schema": "bonsai.bx13-historical-compatibility/v1", "runs": results,
          "archive_sha256": hashlib.sha256(ARCHIVE.read_bytes()).hexdigest(),
          "operator_sha256": hashlib.sha256(BINARY.read_bytes()).hexdigest()}
(OUTPUT / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps({"batch": OUTPUT.relative_to(ROOT).as_posix(), "verified_runs": len(results)}))
