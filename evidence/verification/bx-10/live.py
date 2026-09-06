"""Run both source-pinned learners under one supervisor and identical causal-world policies."""
from __future__ import annotations

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
def main() -> None:
    smoke = sys.argv[1:] == ["smoke"]
    if sys.argv[1:] and not smoke:
        raise SystemExit("expected optional smoke")
    membership = next(line[3:] for line in Path("/proc/self/cgroup").read_text().splitlines()
                      if line.startswith("0::"))
    authority = Path("/sys/fs/cgroup") / str(Path(membership).parent).lstrip("/")
    assert authority.name.startswith("bonsai-bx10-"), authority
    batch = ROOT / "target" / ("bx10-live-" + str(time.time_ns()))
    batch.mkdir()
    base = json.loads((ROOT / "fixtures/experiment-manifest/v1/valid.json").read_text())
    spec = json.loads((ROOT / "fixtures/scenario/causal-v1/spec.json").read_text())
    patch = subprocess.check_output(["git", "diff", "HEAD", "--binary"], cwd=ROOT, stderr=subprocess.DEVNULL)
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    base["source"] = {"repository": "https://github.com/USS-Parks/BONSAI-Research-Labs",
                      "revision": revision, "dirty": True, "dirty_patch_sha256": hashlib.sha256(patch).hexdigest()}
    base["publication_eligibility"] = {
        "status": "ineligible", "reason_codes": ["DIRTY_SOURCE", "VIRTUALIZED_DIAGNOSTIC"]}
    base["environment"] = {"component_id": "bonsai-causal-environment", "version": "1.0.0",
                           "entrypoint": [str(PYTHON), "-I", "-B", str(ROOT / "scripts/adapter_entrypoint.py"),
                                          "environment_adapter"], "config": spec}
    base["scenario"] = {"scenario_id": spec["scenario_id"], "version": spec["version"],
                        "family": "causal-ring", "variant": "paired-conformance",
                        "reward_unit_id": "scenario_reward", "config": spec}
    base["resource_profile"]["profile_id"] = "custom"
    base["resource_profile"]["step_limit"] = 20 if smoke else 200
    rows = []
    operator_hash = hashlib.sha256(BINARY.read_bytes()).hexdigest()
    for seed in ([42] if smoke else [7, 19, 42, 73, 91]):
        for trial in range(1 if smoke else 2):
            for name, identity, version, module, config, touches in [
                ("tabular", "bonsai-primitive-tabular", "1.1.0", "primitive_adapter",
                 {"action_count": 3}, 2),
                ("linear", "bonsai-linear-nlms", "1.0.0", "linear_adapter",
                 {"action_count": 3, "observation_width": spec["observation_width"]}, spec["observation_width"] + 1),
            ]:
                case = batch / f"{name}-s{seed}-t{trial}"
                case.mkdir()
                manifest = json.loads(json.dumps(base))
                manifest["adapter"] = {
                    "component_id": identity, "version": version, "config": config,
                    "entrypoint": [str(PYTHON), "-I", "-B", str(ROOT / "scripts/adapter_entrypoint.py"), module],
                    "accounting_contract": {"schema": "bonsai.online-accounting/v1",
                                            "parameter_touches_per_update": touches,
                                            "parameter_update_schema": "bonsai.parameter-update/v1"}}
                manifest["seeds"] = [{"seed_id": "environment", "value": str(seed)}]
                manifest["manifest_id"], manifest["run_id"] = str(uuid.uuid4()), str(uuid.uuid4())
                path = case / "manifest.json"
                path.write_text(json.dumps(manifest, indent=2) + "\n")
                started = time.monotonic_ns()
                run = subprocess.run([str(BINARY), "run", "--manifest", str(path), "--output", str(case / "run"),
                                      "--authority", str(authority)], cwd=ROOT, capture_output=True, timeout=150)
                (case / "operator.stdout.txt").write_bytes(run.stdout)
                (case / "operator.stderr.txt").write_bytes(run.stderr)
                row = {"agent": name, "seed": seed, "trial": trial, "case": case.relative_to(ROOT).as_posix(),
                       "runner_exit": run.returncode, "elapsed_ns": time.monotonic_ns() - started}
                print(json.dumps(row), flush=True)
                if run.returncode:
                    raise SystemExit(run.stderr.decode())
                receipt = json.loads(run.stdout)["receipt_sha256"]
                verify = subprocess.run([str(BINARY), "verify-run", "--root", str(case / "run/observer"),
                                         "--receipt-sha256", receipt], cwd=ROOT, capture_output=True, timeout=60)
                (case / "verifier.stdout.txt").write_bytes(verify.stdout)
                (case / "verifier.stderr.txt").write_bytes(verify.stderr)
                row.update({"receipt_sha256": receipt, "verifier_exit": verify.returncode})
                if verify.returncode:
                    raise SystemExit(verify.stderr.decode())
                rows.append(row)
                assert hashlib.sha256(BINARY.read_bytes()).hexdigest() == operator_hash
    if not smoke:
        utility = ROOT / "target/x86_64-unknown-linux-gnu/debug/examples/adapter_certification"
        for name, module, config in [
            ("tabular", "primitive_adapter", {"action_count": 3}),
            ("linear", "linear_adapter", {"action_count": 3, "observation_width": spec["observation_width"]}),
        ]:
            configuration = batch / (name + "-timeout-config.json")
            configuration.write_text(json.dumps(config))
            subprocess.run([str(utility), "timeout", str(PYTHON), module, str(configuration),
                            str(batch / ("timeout-" + name))], cwd=ROOT, check=True, timeout=30)
    result = {"schema": "bonsai.paired-learners/v1", "smoke": smoke,
              "operator_sha256": operator_hash, "runs": rows, "resource_profile": base["resource_profile"],
              "scenario": spec}
    (batch / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"batch": batch.relative_to(ROOT).as_posix(), "runs": len(rows),
                      "operator_sha256": operator_hash}), flush=True)
if __name__ == "__main__":
    main()
