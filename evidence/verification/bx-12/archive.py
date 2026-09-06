"""Archive source-bound Windows/Linux feature diagnostics and their independent reconstruction."""
import hashlib
import json
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / "evidence/verification/bx-12"
assert len(sys.argv) == 3
payloads = {}
sources = None
run_count = steps = 0
for argument in sys.argv[1:]:
    batch = (ROOT / argument).resolve()
    assert batch.is_relative_to(ROOT / "target")
    identity = json.loads((batch / "source-identity.json").read_bytes())
    summary = json.loads((batch / "summary.json").read_bytes())
    replay = json.loads((batch / "reconstruction.json").read_bytes())
    assert len(summary["runs"]) == len(replay["runs"]) == 9
    assert replay["result"] == "pass" and replay["reconstructed_steps"] == 1152
    assert len(replay["tampering_rejected"]) == 8
    assert all(row["governor_reaped"] and row["governor_returncode"] == 0 for row in summary["runs"])
    if sources is None:
        sources = identity["source_files"]
    else:
        assert sources == identity["source_files"], "different source snapshots"
    for relative, expected in identity["source_files"].items():
        raw = (ROOT / relative).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == expected, relative
        payloads["source/" + relative] = raw
    for path in sorted(batch.rglob("*")):
        if path.is_file():
            assert path.stat().st_size <= 16 * 1024 * 1024
            payloads[path.relative_to(ROOT).as_posix()] = path.read_bytes()
    run_count += len(summary["runs"])
    steps += replay["reconstructed_steps"]
for path in sorted((ROOT / "target/bx12-audit-proto").rglob("*_pb2.py")):
    payloads["audit-bindings/" + path.relative_to(ROOT / "target/bx12-audit-proto").as_posix()] = path.read_bytes()
manifest = {name: {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
            for name, raw in sorted(payloads.items())}
archive = BASE / "online-features.zip"
with zipfile.ZipFile(archive, "x", zipfile.ZIP_DEFLATED) as output:
    for name, raw in sorted(payloads.items()):
        output.writestr(name, raw)
    output.writestr("manifest.json", json.dumps(manifest, indent=2) + "\n")
with zipfile.ZipFile(archive) as check:
    assert check.testzip() is None
    assert len(check.namelist()) == len(set(check.namelist())) == len(payloads) + 1
    for name, entry in manifest.items():
        raw = check.read(name)
        assert len(raw) == entry["bytes"] and hashlib.sha256(raw).hexdigest() == entry["sha256"]
print(json.dumps({"archive": archive.relative_to(ROOT).as_posix(), "bytes": archive.stat().st_size,
                  "sha256": hashlib.sha256(archive.read_bytes()).hexdigest(), "payloads": len(payloads),
                  "source_files": len(sources), "verified_runs": run_count,
                  "independently_reconstructed_steps": steps}, indent=2))
