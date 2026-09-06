"""BX-08 real-process memory and crash acceptance; retains every raw outcome."""
from __future__ import annotations

import hashlib
import json
import platform
import statistics
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BINARY = ROOT / (
    "target/debug/examples/durable_probe.exe"
    if platform.system() == "Windows"
    else "target/x86_64-unknown-linux-gnu/debug/examples/durable_probe"
)
SIZES = (1000, 10_000, 100_000)
BOUNDARIES = (
    "before_segment_commit", "after_segment_commit",
    "before_index_commit", "after_index_commit",
)


def digest(path: Path) -> str:
    return hashlib.file_digest(path.open("rb"), "sha256").hexdigest()


def source_hashes() -> dict[str, str]:
    paths = [
        p for p in (ROOT / "crates").rglob("*")
        if p.is_file() and p.suffix in {".rs", ".toml", ".sql"}
    ]
    paths += [ROOT / "Cargo.lock", ROOT / "Cargo.toml", Path(__file__).resolve()]
    return {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(paths)}


def run(batch: Path, name: str, operation: str, run_root: Path, value: str, code: int = 0) -> dict:
    result = subprocess.run(
        [str(BINARY), operation, str(run_root), value],
        cwd=ROOT, capture_output=True, timeout=180, check=False,
    )
    (batch / f"{name}.stdout.txt").write_bytes(result.stdout)
    (batch / f"{name}.stderr.txt").write_bytes(result.stderr)
    assert result.returncode == code, (name, result.returncode, result.stderr.decode(errors="replace"))
    return json.loads(result.stdout) if code == 0 else {"exit_code": code}


def main() -> None:
    batch = ROOT / "target" / f"bx08-live-{time.time_ns()}"
    batch.mkdir()
    sources = source_hashes()
    trials = []
    for size in SIZES:
        for trial in range(3):
            name = f"history-{size}-{trial}"
            run_root = batch / name
            growth = run(batch, name, "grow", run_root, str(size))
            recovery = run(batch, name + "-recover", "recover", run_root, "0")
            assert growth["checkpoint"]["events"] == size
            assert growth["live_artifact_cap"] == 1
            assert growth["maximum_batch_events"] == 1000
            assert recovery["report"]["committed_events"] == size
            assert recovery["report"]["orphan_batches"] == 0
            assert recovery["checkpoint"]["continuation"] is True
            for measured in (growth, recovery):
                assert measured["rss_before_bytes"] > 0
                assert measured["rss_after_bytes"] > 0
                assert measured["rss_after_bytes"] - measured["rss_before_bytes"] < 32 * 1024 * 1024
            trials.append({"size": size, "trial": trial, "directory": name, "growth": growth, "recovery": recovery})
            print(json.dumps({"completed": name}), flush=True)
    medians = [
        {
            "size": size,
            "rss_after_bytes": statistics.median(
                t["growth"]["rss_after_bytes"] for t in trials if t["size"] == size
            ),
            "recovery_rss_after_bytes": statistics.median(
                t["recovery"]["rss_after_bytes"] for t in trials if t["size"] == size
            ),
            "growth_ns": statistics.median(t["growth"]["elapsed_ns"] for t in trials if t["size"] == size),
            "recovery_ns": statistics.median(t["recovery"]["elapsed_ns"] for t in trials if t["size"] == size),
        }
        for size in SIZES
    ]
    assert max(m["rss_after_bytes"] for m in medians) - min(m["rss_after_bytes"] for m in medians) < 16 * 1024 * 1024
    recovered_rss = [m["recovery_rss_after_bytes"] for m in medians]
    assert max(recovered_rss) - min(recovered_rss) < 16 * 1024 * 1024
    crashes = []
    for boundary in BOUNDARIES:
        name = "crash-" + boundary
        run_root = batch / name
        run(batch, name, "crash", run_root, boundary, 73)
        recovered = run(batch, name + "-recover", "recover", run_root, "0")
        expected = 2 if boundary == "after_index_commit" else 1
        assert recovered["report"]["committed_events"] == expected
        assert recovered["report"]["orphan_batches"] == (0 if expected == 2 else 1)
        assert recovered["checkpoint"]["continuation"] is True
        crashes.append({"boundary": boundary, "directory": name, "recovered": recovered})
    assert source_hashes() == sources, "measured source changed during verification"
    summary = {
        "format": "bonsai.durable-lineage-acceptance/v1",
        "platform": platform.platform(), "binary_sha256": digest(BINARY),
        "source_hashes": sources, "trials": trials, "medians": medians, "crashes": crashes,
        "rss_scope": "current resident snapshots, not peak; fresh processes and one live artifact",
        "memory_gate": "median spread under 16 MiB and per-process growth under 32 MiB",
    }
    (batch / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"batch": str(batch.relative_to(ROOT)), "medians": medians, "crashes": len(crashes)}))


if __name__ == "__main__":
    main()
