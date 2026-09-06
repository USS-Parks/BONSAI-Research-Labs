"""Verify publication bytes against the Git index (or the committed HEAD)."""

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
head = "--head" in sys.argv
listing = ["git", "diff-tree", "--no-commit-id", "--name-only", "-r", "-z", "HEAD"] if head else [
    "git", "diff", "--cached", "--name-only", "-z"]
paths = subprocess.check_output(listing, cwd=ROOT).decode().split("\0")
paths = [path for path in paths if path]
assert paths, "no publication files"
prefix = "HEAD:" if head else ":"
def blob(path):
    return subprocess.check_output(["git", "cat-file", "blob", prefix + path], cwd=ROOT)

for path in paths:
    assert blob(path) == (ROOT / path).read_bytes(), "Git byte conversion: " + path
records = [json.loads(line) for line in blob("evidence/verification/records.jsonl").decode().splitlines()]
outputs = 0
for record in records:
    if record["prompt"].startswith("BX-07"):
        for stream in ["stdout", "stderr"]:
            reference = record[stream]
            data = blob("evidence/verification/" + reference["path"])
            assert len(data) == reference["bytes"]
            assert hashlib.sha256(data).hexdigest() == reference["sha256"]
            outputs += 1
archive_record = next(record for record in reversed(records) if record["prompt"] == "BX-07-ARCHIVE")
archive = json.loads(blob("evidence/verification/" + archive_record["stdout"]["path"]))
data = blob("evidence/verification/bx-07/scaling-evidence.zip")
assert hashlib.sha256(data).hexdigest() == archive["sha256"] and len(data) == archive["bytes"]
print(json.dumps({"mode": "HEAD" if head else "index", "files": len(paths), "captured_outputs": outputs,
                  "archive_sha256": archive["sha256"], "byte_mismatches": 0}, indent=2))
