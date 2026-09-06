"""Measure bounded BX-07 workloads in fresh processes, one at a time."""
import hashlib
import json
import platform
import statistics
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BINARY = ROOT / ("target/debug/examples/incremental_scaling.exe" if sys.platform == "win32"
                 else "target/x86_64-unknown-linux-gnu/debug/examples/incremental_scaling")
BATCH = ROOT / "target" / ("bx07-scaling-" + str(time.time_ns()))
BATCH.mkdir()
SOURCES = [
    "crates/bonsai-contracts/src/lineage.rs", "crates/bonsai-contracts/src/lineage/incremental.rs",
    "crates/bonsai-lineage/src/lib.rs", "crates/bonsai-lineage/src/graph.rs",
    "crates/bonsai-lineage/tests/incremental.rs", "crates/bonsai-lineage/examples/incremental_scaling.rs",
    "crates/bonsai-lineage/Cargo.toml", "Cargo.lock", "evidence/verification/bx-07/scaling.py",
]


def hashes():
    return {name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest() for name in SOURCES}


identity = {
    "revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
    "tracked_patch_sha256": hashlib.sha256(subprocess.check_output(
        ["git", "diff", "HEAD", "--binary"], cwd=ROOT, stderr=subprocess.DEVNULL)).hexdigest(),
    "source_files": hashes(), "binary_sha256": hashlib.sha256(BINARY.read_bytes()).hexdigest(),
    "platform": platform.platform(), "scope": "local diagnostic; not a physical-platform acceptance claim",
}
results = []
for case in ["history", "roots", "chain", "branch"]:
    for size in [1000, 10000, 100000]:
        trials = []
        for trial in range(3):
            began = time.monotonic_ns()
            result = subprocess.run([str(BINARY), case, str(size)], cwd=ROOT,
                                    capture_output=True, timeout=120, check=False)
            stem = f"{case}-{size}-{trial}"
            (BATCH / (stem + ".stdout.txt")).write_bytes(result.stdout)
            (BATCH / (stem + ".stderr.txt")).write_bytes(result.stderr)
            assert result.returncode == 0, (stem, result.stderr.decode())
            value = json.loads(result.stdout)
            value["external_elapsed_ns"] = time.monotonic_ns() - began
            work = value["validation_work"]
            assert work["events"] == 1 and work["whole_graph_scans"] == work["graph_clones"] == 0
            assert value["prefix_events"] == size and value["case"] == case
            assert value["probe_accepted"] == (case != "chain")
            assert value["rss_after_build_bytes"] > 0 and value["rss_after_probe_bytes"] > 0
            if case == "history":
                assert work["artifact_lookups"] == 2 and work["history_lookups"] == 1
                assert work["ancestry_nodes"] == work["ancestry_edges"] == work["parent_references"] == 0
            elif case == "roots":
                assert work["artifact_lookups"] == work["revision_lookups"] == 1
                assert work["ancestry_nodes"] == work["ancestry_edges"] == work["parent_references"] == 0
            elif case == "chain":
                assert work["ancestry_nodes"] == size and work["ancestry_edges"] == size - 1
                assert value["probe_error"] == "ARTIFACT_LINEAGE_CYCLE"
            else:
                assert work["ancestry_nodes"] == size - 1 and work["ancestry_edges"] == 2 * size - 5
            trials.append(value)
        record = {"case": case, "prefix_events": size, "trials": trials,
                  "median_probe_ns": statistics.median(item["probe_ns"] for item in trials),
                  "median_build_ns": statistics.median(item["build_ns"] for item in trials),
                  "median_rss_after_build_bytes": statistics.median(item["rss_after_build_bytes"] for item in trials),
                  "pass": True}
        results.append(record)
        print(json.dumps({key: value for key, value in record.items() if key != "trials"}), flush=True)
assert hashes() == identity["source_files"], "source changed during measurements"
(BATCH / "summary.json").write_text(json.dumps(
    {"format": "bonsai.lineage-scaling-corpus/v1", "identity": identity, "results": results}, indent=2) + "\n")
print("retained scaling corpus: " + str(BATCH), flush=True)
