"""Generate fresh governed evidence and verify it in a separate process."""
import hashlib
import json
import os
import signal
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
assert authority.name.startswith("bonsai-bx08-interrupt-"), "requires the dedicated delegated unit"
batch = ROOT / "target" / ("bx08-interrupted-" + str(time.time_ns()))
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
segment = batch / "run/observer/telemetry/segment-00000000000000000000.open"
with (batch / "operator.stdout.txt").open("wb") as stdout, (batch / "operator.stderr.txt").open("wb") as stderr:
    process = subprocess.Popen(command, cwd=ROOT, stdout=stdout, stderr=stderr, start_new_session=True)
    try:
        deadline = time.monotonic() + 90
        while time.monotonic() < deadline:
            if segment.exists() and segment.stat().st_size >= 65536:
                break
            if process.poll() is not None:
                raise AssertionError(f"operator exited before interruption: {process.returncode}")
            time.sleep(0.025)
        else:
            raise AssertionError("operator never reached durable telemetry")
    finally:
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=10)
        for owned_group in authority.iterdir():
            if owned_group.is_dir() and owned_group.name != "supervisor":
                kill_file = owned_group / "cgroup.kill"
                if kill_file.exists():
                    kill_file.write_text("1")
assert process.returncode == -signal.SIGKILL
before = hashlib.file_digest(segment.open("rb"), "sha256").hexdigest()
status = json.loads((batch / "run/observer/run-status.json").read_text())
assert status["status"] == "INCOMPLETE"
deadline = time.monotonic() + 5
while True:
    groups = {
        str(p.relative_to(authority)): p.read_text().split()
        for p in authority.rglob("cgroup.procs")
        if "supervisor" not in p.relative_to(authority).parts
    }
    if not any(groups.values()):
        break
    assert time.monotonic() < deadline, groups
    time.sleep(0.05)
result = subprocess.run(
    [str(BINARY), "recover-run", "--root", str(batch / "run/observer"),
     "--output", str(batch / "recovered"), "--maximum-output-bytes", "16777216"],
    cwd=ROOT, capture_output=True, timeout=60, check=False,
)
(batch / "recovery.stdout.txt").write_bytes(result.stdout)
(batch / "recovery.stderr.txt").write_bytes(result.stderr)
assert result.returncode == 0, result.stderr.decode()
report = json.loads(result.stdout)
assert report["status"] == "INCOMPLETE" and report["continuation"] is True
assert report["uninterrupted_track_a_eligible"] is False
assert report["agent_state_restored"] is False
assert report["recovered_events"] > 0
assert report["source_capture_sha256"] == before
assert hashlib.file_digest(segment.open("rb"), "sha256").hexdigest() == before
raw = segment.read_bytes()
offset, frames = 60, 0
while offset + 12 <= len(raw) and raw[offset:offset + 8] == b"BNSFRM01":
    length = int.from_bytes(raw[offset + 8:offset + 12], "little")
    end = offset + 12 + length
    if end + 32 > len(raw):
        break
    assert hashlib.sha256(raw[offset + 12:end]).digest() == raw[end:end + 32]
    frames += 1
    offset = end + 32
assert frames == report["recovered_events"]
verification = subprocess.run(
    [str(BINARY), "verify-run", "--root", str(batch / "recovered"),
     "--receipt-sha256", "0" * 64], cwd=ROOT, capture_output=True, timeout=30, check=False,
)
(batch / "promotion.stdout.txt").write_bytes(verification.stdout)
(batch / "promotion.stderr.txt").write_bytes(verification.stderr)
assert verification.returncode != 0
summary = {
    "format": "bonsai.interrupted-governed-run/v1", "operator_exit": process.returncode,
    "source_sha256": before, "independently_counted_frames": frames,
    "owned_descendant_groups_after_kill": groups, "recovery": report,
    "positive_verification_exit": verification.returncode,
}
(batch / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps({"batch": str(batch.relative_to(ROOT)), "summary": summary}))
