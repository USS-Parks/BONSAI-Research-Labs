"""Independently reconcile actual isolated-filesystem exhaustion evidence."""
import hashlib
import json
import sqlite3
import sys
from pathlib import Path

base = Path(sys.argv[1])
error = (base / "grow.stderr.txt").read_text()
assert "database or disk is full" in error or "No space left on device" in error
assert (base / "exit-code.txt").read_text().strip() == "1"
report = json.loads((base / "recover.stdout.txt").read_text())
events = report["report"]["committed_events"]
assert 0 < events < 100_000 and events % 1000 == 0
assert report["report"]["continuation"] is True
assert report["report"]["orphan_batches"] == 1
with sqlite3.connect(base / "retained" / "lineage.sqlite3") as connection:
    committed = connection.execute("SELECT events FROM metadata").fetchone()[0]
    state = json.loads(connection.execute("SELECT state FROM artifacts").fetchone()[0])
assert committed == events and state["next_sequence"] == events + 1
assert (base / "retained" / "execution-status.txt").read_text().startswith("INCOMPLETE")
sentinel = (base / "user-data.txt").read_bytes()
assert sentinel == b"preserve-user-data\n"
summary = {
    "format": "bonsai.disk-full-proof/v1", "filesystem_cap_bytes": 2 * 1024 * 1024,
    "actual_error": error.strip(), "committed_events": events,
    "recovery": report, "sentinel_sha256": hashlib.sha256(sentinel).hexdigest(),
    "scope": "private user/mount namespace tmpfs on WSL2; source copied before namespace teardown",
}
(base / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary))
