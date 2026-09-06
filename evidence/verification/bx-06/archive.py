"""Archive final BX-06 live runs, rejection corpus, and exact S-run source."""
import hashlib
import json
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
smoke, full, corpus, destination = map(Path, sys.argv[1:])
identity = json.loads((full / "run/observer/source-identity.json").read_bytes())
assert json.loads((full / "verifier.stdout.txt").read_bytes())["claims"]["rows"][0]["c1"] == "pass"
assert all(item["pass"] for item in json.loads((corpus / "summary.json").read_bytes()))
files = {}
for relative, expected in identity["source_files"].items():
    path = (ROOT / relative).resolve()
    assert path.is_relative_to(ROOT.resolve()) and path.is_file()
    data = path.read_bytes()
    assert hashlib.sha256(data).hexdigest() == expected, relative
    files["source/" + relative] = data
for label, directory in [("smoke", smoke), ("S", full), ("negative-corpus", corpus)]:
    for path in sorted(directory.rglob("*")):
        assert not path.is_symlink()
        if path.is_file():
            files[label + "/" + path.relative_to(directory).as_posix()] = path.read_bytes()
for path in sorted(Path(__file__).parent.iterdir()):
    if path.suffix in {".py", ".sh", ".cmd", ".md"}:
        files["verification/" + path.name] = path.read_bytes()
manifest = {
    "format": "bonsai.verification-archive/v1",
    "scope": "BX-06 real smoke/S runs, fresh-process negative corpus, exact S source; no binary duplication",
    "operator_executable_sha256": identity["operator_executable_sha256"],
    "files": [{"path": path, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
              for path, data in sorted(files.items())],
}
files["archive-manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
destination.parent.mkdir(parents=True, exist_ok=True)
with zipfile.ZipFile(destination, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
    for path, data in sorted(files.items()):
        archive.writestr(path, data)
with zipfile.ZipFile(destination) as archive:
    assert archive.testzip() is None
    for item in manifest["files"]:
        assert hashlib.sha256(archive.read(item["path"])).hexdigest() == item["sha256"]
print(json.dumps({"path": str(destination), "sha256": hashlib.sha256(destination.read_bytes()).hexdigest(),
                  "bytes": destination.stat().st_size, "archived_files": len(files),
                  "source_files": len(identity["source_files"]), "verification": "all entries re-read and hashed"},
                 indent=2))
