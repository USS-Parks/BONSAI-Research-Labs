"""Preserve BX-07 measured trials and their exact implementation sources."""
import hashlib
import json
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
windows, linux, destination = map(Path, sys.argv[1:])
datasets = {name: json.loads((directory / "summary.json").read_bytes())
            for name, directory in [("windows", windows), ("linux", linux)]}
expected = datasets["windows"]["identity"]["source_files"]
assert datasets["linux"]["identity"]["source_files"] == expected
files = {}
for name, expected_hash in expected.items():
    data = (ROOT / name).read_bytes()
    assert hashlib.sha256(data).hexdigest() == expected_hash, name
    files["source/" + name] = data
for name, directory in [("windows", windows), ("linux", linux)]:
    assert all(record["pass"] and len(record["trials"]) == 3 for record in datasets[name]["results"])
    assert len(datasets[name]["results"]) == 12
    for path in sorted(directory.iterdir()):
        assert path.is_file() and not path.is_symlink()
        files[name + "/" + path.name] = path.read_bytes()
manifest = {"format": "bonsai.verification-archive/v1",
            "scope": "72 fresh-process scaling trials, two source/binary identities, exact measured sources",
            "files": [{"path": name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}
                      for name, data in sorted(files.items())]}
files["archive-manifest.json"] = (json.dumps(manifest, indent=2) + "\n").encode()
with zipfile.ZipFile(destination, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
    for name, data in sorted(files.items()):
        archive.writestr(name, data)
with zipfile.ZipFile(destination) as archive:
    assert archive.testzip() is None
    for item in manifest["files"]:
        assert hashlib.sha256(archive.read(item["path"])).hexdigest() == item["sha256"]
print(json.dumps({"path": str(destination), "bytes": destination.stat().st_size,
                  "sha256": hashlib.sha256(destination.read_bytes()).hexdigest(),
                  "archived_files": len(files), "source_files": len(expected),
                  "trials": 72, "verification": "all entries re-read and hashed"}, indent=2))
