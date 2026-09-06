"""Archive a verified live batch with its exact implementation source bytes."""

import hashlib
import json
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
batch, destination = Path(sys.argv[1]), Path(sys.argv[2])
summary = json.loads((batch / "summary.json").read_text())
assert summary and all(item["pass"] for item in summary)
identity = json.loads((batch / "s_profile/observer/source-identity.json").read_text())
files = {}
for relative, expected in identity["source_files"].items():
    path = (ROOT / relative).resolve()
    assert path.is_relative_to(ROOT.resolve()) and path.is_file()
    data = path.read_bytes()
    assert hashlib.sha256(data).hexdigest() == expected, relative
    files["source/" + relative] = data
for path in sorted(batch.rglob("*")):
    assert not path.is_symlink()
    if path.is_file():
        files["runs/" + path.relative_to(batch).as_posix()] = path.read_bytes()
for path in sorted(Path(__file__).parent.iterdir()):
    if path.suffix in {".py", ".sh", ".cmd", ".md"}:
        files["verification/" + path.name] = path.read_bytes()
archive_manifest = {
    "format": "bonsai.verification-archive/v1",
    "scope": "live batch plus source bytes verified against its S run identity; no operator binary duplication",
    "operator_executable_sha256": identity["operator_executable_sha256"],
    "files": [{"path": path, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
              for path, data in sorted(files.items())],
}
files["archive-manifest.json"] = (json.dumps(archive_manifest, indent=2) + "\n").encode()
destination.parent.mkdir(parents=True, exist_ok=True)
with zipfile.ZipFile(destination, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
    for path, data in sorted(files.items()):
        archive.writestr(path, data)
    for path in sorted(batch.rglob("*")):
        if path.is_dir() and not any(path.iterdir()):
            archive.writestr("runs/" + path.relative_to(batch).as_posix() + "/", b"")
with zipfile.ZipFile(destination) as archive:
    assert archive.testzip() is None
    for item in archive_manifest["files"]:
        assert hashlib.sha256(archive.read(item["path"])).hexdigest() == item["sha256"]
print(json.dumps({"path": str(destination), "sha256": hashlib.sha256(destination.read_bytes()).hexdigest(),
                  "bytes": destination.stat().st_size, "archived_files": len(files),
                  "source_files": len(identity["source_files"]),
                  "verification": "all archive entries re-read and hashed"},
                 indent=2))
