"""Generate fresh governed evidence and verify it in a separate process."""
import hashlib
import json
import subprocess
import sys
import time
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BINARY = ROOT / "target/x86_64-unknown-linux-gnu/debug/bonsai-xtask"
PYTHON = ROOT / "target/linux-venv/bin/python"
membership = next(line[3:] for line in Path("/proc/self/cgroup").read_text().splitlines() if line.startswith("0::"))
authority = Path("/sys/fs/cgroup") / str(Path(membership).parent).lstrip("/")
batch = ROOT / "target" / ("bx06-live-" + str(time.time_ns()))
batch.mkdir()
manifest = json.loads((ROOT / "fixtures/experiment-manifest/v1/valid.json").read_text())
spec = json.loads((ROOT / "fixtures/scenario/causal-v1/spec.json").read_text())
patch = subprocess.check_output(["git", "diff", "HEAD", "--binary"], cwd=ROOT, stderr=subprocess.DEVNULL)
manifest["source"] = {"repository": "https://github.com/USS-Parks/BONSAI-Research-Labs",
    "revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
    "dirty": True, "dirty_patch_sha256": hashlib.sha256(patch).hexdigest()}
manifest["publication_eligibility"] = {
    "status": "ineligible", "reason_codes": ["DIRTY_SOURCE", "VIRTUALIZED_DIAGNOSTIC"]}
for key, identity, module, configuration in [
    ("adapter", "bonsai-primitive-tabular", "primitive_adapter", {"action_count": 3}),
    ("environment", "bonsai-causal-environment", "environment_adapter", spec),
]:
    manifest[key] = {"component_id": identity, "version": "1.0.0",
        "entrypoint": [str(PYTHON), "-I", "-B", str(ROOT / "scripts/adapter_entrypoint.py"), module],
        "config": configuration}
manifest["scenario"] = {"scenario_id": spec["scenario_id"], "version": spec["version"],
    "family": "causal-ring", "variant": "smoke", "reward_unit_id": "scenario_reward", "config": spec}
manifest["seeds"] = [{"seed_id": "environment", "value": "42"}]
manifest["manifest_id"], manifest["run_id"] = str(uuid.uuid4()), str(uuid.uuid4())
if sys.argv[1:] == ["smoke"]:
    manifest["resource_profile"]["profile_id"] = "custom"
    manifest["resource_profile"]["step_limit"] = 20
elif sys.argv[1:]:
    raise SystemExit("expected optional smoke")
path = batch / "manifest.json"
path.write_text(json.dumps(manifest, indent=2) + "\n")
command = [str(BINARY), "run", "--manifest", str(path), "--output", str(batch / "run"), "--authority", str(authority)]
started = time.monotonic_ns()
run = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=150, check=False)
(batch / "operator.stdout.txt").write_bytes(run.stdout)
(batch / "operator.stderr.txt").write_bytes(run.stderr)
print(json.dumps({"batch": str(batch), "runner_exit": run.returncode,
    "elapsed_ns": time.monotonic_ns() - started}), flush=True)
if run.returncode:
    raise SystemExit(run.returncode)
receipt = json.loads(run.stdout)["receipt_sha256"]
verify = subprocess.run([str(BINARY), "verify-run", "--root", str(batch / "run/observer"),
    "--receipt-sha256", receipt], cwd=ROOT, capture_output=True, timeout=60, check=False)
(batch / "verifier.stdout.txt").write_bytes(verify.stdout)
(batch / "verifier.stderr.txt").write_bytes(verify.stderr)
print(verify.stdout.decode(), end="")
print(verify.stderr.decode(), end="", file=sys.stderr)
print(json.dumps({"batch": str(batch), "verifier_exit": verify.returncode}), flush=True)
raise SystemExit(verify.returncode)
