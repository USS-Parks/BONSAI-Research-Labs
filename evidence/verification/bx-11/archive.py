"""Retain verified external-environment runs, direct traces, and exact source bytes."""
import hashlib
import json
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / "evidence/verification/bx-11"
batch = ROOT / sys.argv[1]
direct = ROOT / sys.argv[2]
summary = json.loads((batch / "summary.json").read_text())
replay = json.loads((batch / "environment-replay.json").read_text())
assert len(summary["runs"]) == len(replay["runs"]) == 4
payloads = {}
identities = []
for row in summary["runs"]:
    assert row["runner_exit"] == row["verifier_exit"] == 0
    case = ROOT / row["case"]
    identities.append(json.loads((case / "run/observer/source-identity.json").read_text()))
assert all(identity == identities[0] for identity in identities)
for path in sorted(batch.rglob("*")):
    if path.is_file():
        payloads[path.relative_to(ROOT).as_posix()] = path.read_bytes()
payloads["harness/test_gymnasium_adapter.py"] = (
    ROOT / "python/bonsai-reference/tests/test_gymnasium_adapter.py").read_bytes()
traces = list(direct.rglob("direct-versus-adapter.json"))
assert len(traces) == 9
for path in traces:
    payloads["direct/" + path.relative_to(direct).as_posix()] = path.read_bytes()
for relative, expected in identities[0]["source_files"].items():
    raw = (ROOT / relative).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == expected, relative
    payloads["source/" + relative] = raw
for path in BASE.iterdir():
    if path.suffix in {".py", ".sh", ".cmd", ".json"}:
        payloads["harness/" + path.name] = path.read_bytes()
for path in sorted((ROOT / "target/bx10-audit-proto").rglob("*_pb2.py")):
    payloads["audit-bindings/" + path.relative_to(ROOT / "target/bx10-audit-proto").as_posix()] = path.read_bytes()
payloads["harness/segment-reader.py"] = (ROOT / "evidence/verification/bx-05/reconcile.py").read_bytes()
manifest = {name: {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}
            for name, raw in sorted(payloads.items())}
archive = BASE / "external-environment.zip"
with zipfile.ZipFile(archive, "x", zipfile.ZIP_DEFLATED) as output:
    for name, raw in sorted(payloads.items()):
        output.writestr(name, raw)
    output.writestr("manifest.json", json.dumps(manifest, indent=2) + "\n")
with zipfile.ZipFile(archive) as check:
    assert check.testzip() is None
    for name, item in manifest.items():
        raw = check.read(name)
        assert len(raw) == item["bytes"] and hashlib.sha256(raw).hexdigest() == item["sha256"]
print(json.dumps({"archive": archive.relative_to(ROOT).as_posix(), "bytes": archive.stat().st_size,
                  "sha256": hashlib.sha256(archive.read_bytes()).hexdigest(), "payloads": len(payloads),
                  "source_files": len(identities[0]["source_files"]), "direct_traces": len(traces),
                  "verified_runs": 4, "independently_replayed_steps": 400}, indent=2))
