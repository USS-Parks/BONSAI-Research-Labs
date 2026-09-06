"""Archive exact BX-08 sources, live trials, failures and compatibility fixture."""
from __future__ import annotations

import hashlib
import json
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    windows, linux, disk, interruption, output = [ROOT / p for p in sys.argv[1:]]
    windows_summary = json.loads((windows / "summary.json").read_text())
    linux_summary = json.loads((linux / "summary.json").read_text())
    assert windows_summary["source_hashes"] == linux_summary["source_hashes"]
    sources = windows_summary["source_hashes"]
    for relative, expected in sources.items():
        assert digest((ROOT / relative).read_bytes()) == expected, relative
    entries: dict[str, Path] = {}
    for label, directory in [
        ("windows", windows), ("linux", linux), ("disk-full", disk),
        ("interrupted-governed-run", interruption),
        ("checkpoint-v1", ROOT / "fixtures/lineage-checkpoint/v1"),
    ]:
        directory.resolve().relative_to(ROOT.resolve())
        for path in sorted(directory.rglob("*")):
            assert not path.is_symlink(), path
            if path.is_file():
                entries[f"{label}/{path.relative_to(directory).as_posix()}"] = path
    for relative in sources:
        entries["source/" + relative] = ROOT / relative
    for path in sorted((ROOT / "evidence/verification/bx-08").glob("*")):
        if path.suffix in {".py", ".sh", ".cmd"}:
            entries["harness/" + path.name] = path
    hashes = {}
    with zipfile.ZipFile(output, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for name, path in entries.items():
            data = path.read_bytes()
            archive.writestr(name, data)
            hashes[name] = {"sha256": digest(data), "bytes": len(data)}
        archive.writestr("manifest.json", json.dumps(hashes, indent=2) + "\n")
    with zipfile.ZipFile(output) as archive:
        assert archive.testzip() is None
        for name, expected in hashes.items():
            data = archive.read(name)
            assert len(data) == expected["bytes"] and digest(data) == expected["sha256"], name
    print(json.dumps({
        "archive": str(output.relative_to(ROOT)), "bytes": output.stat().st_size,
        "sha256": digest(output.read_bytes()), "files": len(hashes),
        "source_files": len(sources), "history_trials": len(windows_summary["trials"]) + len(linux_summary["trials"]),
        "abrupt_exit_cases": len(windows_summary["crashes"]) + len(linux_summary["crashes"]),
    }, indent=2))


if __name__ == "__main__":
    main()
