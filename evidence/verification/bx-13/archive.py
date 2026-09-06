"""Archive the complete verified option matrix, reconstruction and exact source."""
import hashlib
import json
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
assert len(sys.argv) == 4, "archive.py <matrix-batch> <historical-batch> <protocol-batch>"
batch, historical, protocol = [(ROOT / argument).resolve() for argument in sys.argv[1:]]
assert all(path.is_relative_to(ROOT / "target") and path.is_dir() for path in (batch, historical, protocol))
identity = json.loads((batch / "source-identity.json").read_bytes())
summary = json.loads((batch / "summary.json").read_bytes())
replay = json.loads((batch / "option-replay.json").read_bytes())
assert summary["provisional"] is False
assert len(summary["runs"]) == len(replay["runs"]) == 13
assert replay["result"] == "pass" and replay["reconstructed_steps"] == 3328
assert len(replay["tampering_rejected"]) >= 10
assert all(row["runner_exit"] == row["verifier_exit"] == 0 for row in summary["runs"])
assert all(row["resource_measurements"]["children_reaped_and_cgroups_empty"] for row in replay["runs"])
old = json.loads((historical / "summary.json").read_bytes())
assert len(old["runs"]) == 4 and all(row["exit_code"] == 0 for row in old["runs"])
assert old["operator_sha256"] == summary["operator_sha256"]


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


payloads = {}
for relative, expected in identity["source_files"].items():
    path = (ROOT / relative).resolve()
    assert path.is_relative_to(ROOT) and not path.is_symlink()
    assert path.stat().st_size <= 16 * 1024 * 1024 and digest(path) == expected, relative
    payloads["source/" + relative] = path
for directory in (batch, historical):
    for path in sorted(directory.rglob("*")):
        if path.is_file():
            assert not path.is_symlink() and path.stat().st_size <= 64 * 1024 * 1024
            payloads[path.relative_to(ROOT).as_posix()] = path
protocol_summary = json.loads((protocol / "summary.json").read_bytes())
assert protocol_summary["exit_code"] == 0
for path in sorted(protocol.iterdir()):
    if path.is_file():
        payloads[path.relative_to(ROOT).as_posix()] = path
bindings = ROOT / "target/bx13-audit-proto"
for path in sorted(bindings.rglob("*_pb2.py")):
    payloads["audit-bindings/" + path.relative_to(bindings).as_posix()] = path
assert len(payloads) <= 10000
manifest = {name: {"bytes": path.stat().st_size, "sha256": digest(path)}
            for name, path in sorted(payloads.items())}
archive_path = ROOT / "evidence/verification/bx-13/online-options.zip"
with zipfile.ZipFile(archive_path, "x", zipfile.ZIP_DEFLATED) as archive:
    for name, path in sorted(payloads.items()):
        archive.write(path, name)
    archive.writestr("manifest.json", json.dumps(manifest, indent=2) + "\n")
with zipfile.ZipFile(archive_path) as archive:
    assert archive.testzip() is None
    assert len(archive.namelist()) == len(set(archive.namelist())) == len(payloads) + 1
    for name, entry in manifest.items():
        with archive.open(name) as stream:
            assert hashlib.file_digest(stream, "sha256").hexdigest() == entry["sha256"]
        assert archive.getinfo(name).file_size == entry["bytes"]
print(json.dumps({"archive": archive_path.relative_to(ROOT).as_posix(), "bytes": archive_path.stat().st_size,
                  "sha256": digest(archive_path), "payloads": len(payloads),
                  "source_files": len(identity["source_files"]), "verified_runs": 13,
                  "independently_reconstructed_steps": 3328, "historical_runs": 4}, indent=2))
