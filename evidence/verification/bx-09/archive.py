"""Archive verified live adapter compatibility evidence and exact source bytes."""
from __future__ import annotations

import hashlib
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / "evidence/verification/bx-09"
BATCHES = ["target/bx09-live-1788671469838574600", "target/bx09-live-1788671530161085136"]
def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()
def main() -> None:
    summaries = [json.loads((ROOT / batch / "summary.json").read_text()) for batch in BATCHES]
    assert summaries[0]["sources"] == summaries[1]["sources"]
    assert summaries[0]["reports_sha256"] == summaries[1]["reports_sha256"]
    payloads = {}
    for batch in BATCHES:
        for path in sorted((ROOT / batch).rglob("*")):
            if path.is_file():
                payloads[path.relative_to(ROOT).as_posix()] = path.read_bytes()
    for path, expected in summaries[0]["sources"].items():
        raw = (ROOT / path).read_bytes()
        assert sha(raw) == expected, path
        payloads["source/" + path] = raw
    for path in BASE.iterdir():
        if path.suffix in {".py", ".sh", ".cmd", ".json"}:
            payloads["harness/" + path.name] = path.read_bytes()
    manifest = {name: {"bytes": len(raw), "sha256": sha(raw)} for name, raw in sorted(payloads.items())}
    archive = BASE / "compatibility.zip"
    with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as output:
        for name, raw in sorted(payloads.items()):
            output.writestr(name, raw)
        output.writestr("manifest.json", json.dumps(manifest, indent=2) + "\n")
    with zipfile.ZipFile(archive) as check:
        assert check.testzip() is None
        for name, item in manifest.items():
            raw = check.read(name)
            assert len(raw) == item["bytes"] and sha(raw) == item["sha256"]
    result = {"archive": archive.relative_to(ROOT).as_posix(), "bytes": archive.stat().st_size,
              "sha256": sha(archive.read_bytes()), "payloads": len(payloads),
              "source_files": len(summaries[0]["sources"]), "live_certifications": 24,
              "matching_report_sha256": summaries[0]["reports_sha256"]}
    print(json.dumps(result, indent=2))
if __name__ == "__main__":
    main()
