"""Verify retained BX-10 bundles with the current optional-environment verifier."""
import json
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parents[3]
batch = root / "target/bx10-live-1788675720870060478"
summary = json.loads((batch / "summary.json").read_text())
for agent in ["tabular", "linear"]:
    row = next(row for row in summary["runs"] if row["agent"] == agent)
    subprocess.run([str(root / "target/x86_64-unknown-linux-gnu/debug/bonsai-xtask"),
                    "verify-run", "--root", str(root / row["case"] / "run/observer"),
                    "--receipt-sha256", row["receipt_sha256"]], check=True, timeout=60)
print(json.dumps({"historical_bx10_agents_verified": ["tabular", "linear"]}))
