"""Real operator preflight rejection tests; no delegated authority is granted."""

import hashlib
import json
import subprocess
import sys
import time
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BINARY = ROOT / "target/x86_64-unknown-linux-gnu/debug/bonsai-xtask"
BASE_PATH = Path(sys.argv[1])
BASE = json.loads(BASE_PATH.read_text())
PATCH = subprocess.check_output(["git", "diff", "HEAD", "--binary"], cwd=ROOT, stderr=subprocess.DEVNULL)
BASE["source"]["dirty_patch_sha256"] = hashlib.sha256(PATCH).hexdigest()
BASE["source"]["revision"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
BATCH = ROOT / "target" / ("bx05-preflight-" + str(time.time_ns()))
BATCH.mkdir()
results = []
for case, expected in [
    ("revision", "SOURCE_REVISION_MISMATCH"),
    ("patch", "SOURCE_PATCH_MISMATCH"),
    ("profile", "RUN_PROFILE_UNSUPPORTED"),
    ("counter", "REQUIRED_COUNTER_UNSUPPORTED"),
    ("relative_executable", "ABSOLUTE_COMPONENT_EXECUTABLE_REQUIRED"),
]:
    manifest = json.loads(json.dumps(BASE))
    manifest["manifest_id"], manifest["run_id"] = str(uuid.uuid4()), str(uuid.uuid4())
    if case == "revision":
        manifest["source"]["revision"] = "0" * 40
    elif case == "patch":
        manifest["source"]["dirty_patch_sha256"] = "0" * 64
    elif case == "profile":
        manifest["resource_profile"]["profile_id"] = "C"
    elif case == "counter":
        manifest["expected_counters"][0]["counter_id"] = "unsupported_counter"
        manifest["expected_counters"][0]["required_for_run"] = True
    else:
        manifest["adapter"]["entrypoint"][0] = "python"
    path = BATCH / (case + "-manifest.json")
    path.write_text(json.dumps(manifest, indent=2) + "\n")
    output = BATCH / case
    command = [str(BINARY), "run", "--manifest", str(path), "--output", str(output),
               "--authority", str(BATCH / "no-authority-granted")]
    completed = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=40, check=False)
    (BATCH / (case + "-stdout.txt")).write_bytes(completed.stdout)
    (BATCH / (case + "-stderr.txt")).write_bytes(completed.stderr)
    passed = completed.returncode == 1 and expected.encode() in completed.stderr and not output.exists()
    results.append({"case": case, "expected": expected, "exit_code": completed.returncode,
                    "output_created": output.exists(), "pass": passed})
    print(json.dumps(results[-1]), flush=True)
(BATCH / "summary.json").write_text(json.dumps(results, indent=2) + "\n")
print("retained preflight directory: " + str(BATCH), flush=True)
raise SystemExit(0 if all(result["pass"] for result in results) else 1)
