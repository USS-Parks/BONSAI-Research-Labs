import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BATCH = ROOT / "target/bx13-live-1788717713189903230"
summary = json.loads((BATCH / "summary.json").read_bytes())
assert len(summary["runs"]) == 13
binary = ROOT / "target/debug/bonsai-xtask.exe"
rows = []
for row in summary["runs"]:
    observer = ROOT / row["case"] / "run/observer"
    result = subprocess.run(
        [str(binary), "verify-run", "--root", str(observer), "--receipt-sha256", row["receipt_sha256"]],
        cwd=ROOT,
        capture_output=True,
        timeout=60,
    )
    record = {
        "case": row["case"],
        "receipt_sha256": row["receipt_sha256"],
        "exit_code": result.returncode,
        "stdout": result.stdout.decode(),
        "stderr": result.stderr.decode(),
    }
    rows.append(record)
    assert result.returncode == 0, record
report = {
    "schema": "bonsai.bx13-portable-verifier/v1",
    "source_batch": BATCH.relative_to(ROOT).as_posix(),
    "native_verifier_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
    "platform": "windows-x86_64",
    "linux_governed_bundles_verified": 13,
    "rows": rows,
    "scope": "Offline receipt verification across hosts; no new learner, OS-controller or physical acceptance claim.",
}
(ROOT / "evidence/verification/bx-13-portability/native-verification.json").write_text(
    json.dumps(report, indent=2) + "\n"
)
print(
    json.dumps(
        {
            "result": "pass",
            "linux_bundles_verified_on_windows": 13,
            "native_verifier_sha256": report["native_verifier_sha256"],
        },
        indent=2,
    )
)
