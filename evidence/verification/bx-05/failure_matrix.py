"""Exercise real runner failure boundaries within an already delegated user unit."""

import hashlib
import json
import subprocess
import sys
import threading
import time
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BINARY = ROOT / "target/x86_64-unknown-linux-gnu/debug/bonsai-xtask"
PYTHON = ROOT / "target/linux-venv/bin/python"
MEMBERSHIP = next(line[3:] for line in Path("/proc/self/cgroup").read_text().splitlines() if line.startswith("0::"))
AUTHORITY = Path("/sys/fs/cgroup") / str(Path(MEMBERSHIP).parent).lstrip("/")
BATCH = ROOT / "target" / ("bx05-failures-" + str(time.time_ns()))
BATCH.mkdir()
BASE = json.loads((ROOT / "fixtures/experiment-manifest/v1/valid.json").read_text())
SPEC = json.loads((ROOT / "fixtures/scenario/causal-v1/spec.json").read_text())
PATCH = subprocess.check_output(["git", "diff", "HEAD", "--binary"], cwd=ROOT, stderr=subprocess.DEVNULL)
BASE["source"] = {"repository": "https://github.com/USS-Parks/BONSAI-Research-Labs",
                  "revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                  "dirty": True, "dirty_patch_sha256": hashlib.sha256(PATCH).hexdigest()}
BASE["publication_eligibility"] = {"status": "ineligible", "reason_codes": ["DIRTY_SOURCE", "VIRTUALIZED_DIAGNOSTIC"]}
BASE["adapter"] = {"component_id": "bonsai-primitive-tabular", "version": "1.0.0",
                   "entrypoint": [str(PYTHON), "-I", "-B", str(ROOT / "scripts/adapter_entrypoint.py"),
                                  "primitive_adapter"],
                   "config": {"action_count": 3}}
BASE["environment"] = {"component_id": "bonsai-causal-environment", "version": "1.0.0",
                       "entrypoint": [str(PYTHON), "-I", "-B", str(ROOT / "scripts/adapter_entrypoint.py"),
                                      "environment_adapter"],
                       "config": SPEC}
BASE["scenario"] = {"scenario_id": SPEC["scenario_id"], "version": SPEC["version"], "family": "causal-ring",
                    "variant": "smoke", "reward_unit_id": "scenario_reward", "config": SPEC}
BASE["seeds"] = [{"seed_id": "environment", "value": "42"}]
def request_cancellation(run_output, stop, request, pending=False):
    segment = run_output / "observer/telemetry/segment-00000000000000000000.open"
    while not stop.wait(0.02):
        if ((pending and (run_output / "agent/work/pending-initialization").exists())
                or (not pending and segment.exists() and segment.stat().st_size >= 32_768)):
            request.touch()
            return


cases = {"exit", "memory", "storage", "delay", "cancel", "cancel_pending", "wall", "smoke", "observer", "finalization"}
cases.add("s_profile")
selected = set(sys.argv[1:]) or cases
if selected - cases:
    raise SystemExit("unknown failure case")
def prevent_report_write(run_output, stop):
    marker = run_output / "observer/run-status.json"
    while not stop.wait(0.02):
        if marker.exists():
            (run_output / "observer/reports/index.html").mkdir()
            return


results = []
for case, expected in [("exit", "agent:ADAPTER_EOF"), ("memory", "AGENT_RESOURCE_VIOLATION"),
                       ("storage", "AGENT_STORAGE_EXHAUSTED"), ("delay", "agent:TRANSPORT_READ_TIMEOUT"),
                       ("cancel", "RUN_CANCELLED"), ("cancel_pending", "RUN_CANCELLED"),
                       ("wall", "RUN_WALL_LIMIT_EXCEEDED"), ("smoke", "COMPLETE"),
                       ("observer", "OBSERVER_STORAGE_EXHAUSTED"), ("s_profile", "COMPLETE"),
                       ("finalization", "RUN_FINALIZATION_FAILED")]:
    if case not in selected:
        continue
    manifest = json.loads(json.dumps(BASE))
    manifest["manifest_id"], manifest["run_id"] = str(uuid.uuid4()), str(uuid.uuid4())
    if case not in {"cancel", "finalization", "wall", "smoke", "observer", "s_profile"}:
        manifest["adapter"]["entrypoint"] = [str(PYTHON), "-I", "-B",
                                             str(Path(__file__).with_name("fault_adapter.py")), case]
        manifest["resource_profile"]["profile_id"] = "custom"
        manifest["resource_profile"]["step_limit"] = 20
    if case in {"finalization", "wall", "smoke", "observer"}:
        manifest["resource_profile"]["profile_id"] = "custom"
        manifest["resource_profile"]["step_limit"] = 20
    if case == "observer":
        manifest["resource_profile"]["observer_output_limit_bytes"] = 262_144
    if case == "wall":
        manifest["resource_profile"]["wall_time_limit_ns"] = 1
    if case == "memory":
        manifest["resource_profile"]["agent_rss_limit_bytes"] = 64 * 1024 * 1024
    if case == "storage":
        manifest["resource_profile"]["agent_storage_limit_bytes"] = 65_536
    path = BATCH / (case + "-manifest.json")
    path.write_text(json.dumps(manifest, indent=2) + "\n")
    output = BATCH / case
    command = [str(BINARY), "run", "--manifest", str(path), "--output", str(output), "--authority", str(AUTHORITY)]
    timer = None
    cancellation_stop = threading.Event()
    if case in {"cancel", "cancel_pending"}:
        cancellation = BATCH / (case + ".request")
        command.extend(["--cancel-file", str(cancellation)])
        timer = threading.Thread(target=request_cancellation,
                                 args=(output, cancellation_stop, cancellation, case == "cancel_pending"))
        timer.start()
    if case == "finalization":
        timer = threading.Thread(target=prevent_report_write, args=(output, cancellation_stop))
        timer.start()
    began = time.monotonic()
    try:
        completed = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=150, check=False)
    finally:
        if timer is not None:
            cancellation_stop.set()
            timer.join()
    (BATCH / (case + "-stdout.txt")).write_bytes(completed.stdout)
    (BATCH / (case + "-stderr.txt")).write_bytes(completed.stderr)
    if case == "finalization":
        status = json.loads((output / "observer/run-status.json").read_text())
        actual = status["reason_code"]
        passed = completed.returncode != 0 and status["status"] == "INCOMPLETE" and actual == expected
    else:
        report = json.loads((output / "observer/reports/report.json").read_text())
        actual = report["behavior"]["details"].get("reason_code", report["behavior"]["status"])
        passed = (completed.returncode == (0 if case in {"smoke", "s_profile"} else 2)
                  and report["behavior"]["status"] == ("COMPLETE" if case in {"smoke", "s_profile"} else "INCOMPLETE")
                  and actual == expected)
    if case == "cancel":
        segment = output / "observer/telemetry/segment-00000000000000000000.bseg"
        passed = passed and b"run.action" in segment.read_bytes()
    if case == "cancel_pending":
        passed = passed and (output / "agent/work/pending-initialization").exists()
    results.append({"case": case, "expected": expected, "actual": actual, "exit_code": completed.returncode,
                    "elapsed_s": time.monotonic() - began, "pass": passed, "output": str(output)})
    print(json.dumps(results[-1]), flush=True)
(BATCH / "summary.json").write_text(json.dumps(results, indent=2) + "\n")
print("retained diagnostic directory: " + str(BATCH), flush=True)
raise SystemExit(0 if all(result["pass"] for result in results) else 1)
