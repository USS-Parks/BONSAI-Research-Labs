"""Source-bound online option runs under the existing unprivileged supervisor."""
from __future__ import annotations

import argparse
import hashlib
import json
import platform
import subprocess
import time
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BINARY = ROOT / "target/x86_64-unknown-linux-gnu/debug/bonsai-xtask"
PYTHON = ROOT / "target/linux-venv/bin/python"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_snapshot():
    paths = []
    for directory in ["crates", "proto", "schemas", "scripts", "python/bonsai-reference/src",
                      "python/bonsai-reference/tests", "fixtures/online-options", "evidence/verification/bx-13"]:
        paths.extend(path for path in (ROOT / directory).rglob("*")
                     if path.is_file() and "__pycache__" not in path.parts
                     and path.suffix in {".rs", ".py", ".pyi", ".proto", ".json", ".toml", ".lock", ".sh", ".cmd"})
    paths.extend(ROOT / name for name in ["Cargo.toml", "Cargo.lock", "pyproject.toml", "uv.lock",
                                         "rust-toolchain.toml", ".github/workflows/ci.yml", ".gitattributes",
                                         "evidence/verification/bx-05/reconcile.py"])
    assert len(paths) <= 10000 and all(path.stat().st_size <= 16 * 1024 * 1024 for path in paths)
    return {path.relative_to(ROOT).as_posix(): digest(path) for path in sorted(paths)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--provisional", action="store_true")
    args = parser.parse_args()
    membership = next(line[3:] for line in Path("/proc/self/cgroup").read_text().splitlines()
                      if line.startswith("0::"))
    authority = Path("/sys/fs/cgroup") / str(Path(membership).parent).lstrip("/")
    assert authority.name.startswith("bonsai-bx13-"), authority
    batch = ROOT / "target" / ("bx13-live-" + str(time.time_ns()))
    batch.mkdir()
    print(json.dumps({"batch": batch.relative_to(ROOT).as_posix(), "provisional": args.provisional}), flush=True)
    base = json.loads((ROOT / "fixtures/experiment-manifest/v1/valid.json").read_text())
    patch = subprocess.check_output(["git", "diff", "HEAD", "--binary"], cwd=ROOT,
                                    stderr=subprocess.DEVNULL, timeout=120)
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True, timeout=120).strip()
    source = source_snapshot()
    base["source"] = {"repository": "https://github.com/USS-Parks/BONSAI-Research-Labs",
                      "revision": revision, "dirty": True, "dirty_patch_sha256": hashlib.sha256(patch).hexdigest()}
    base["publication_eligibility"] = {
        "status": "ineligible", "reason_codes": ["DIRTY_SOURCE", "VIRTUALIZED_DIAGNOSTIC"]}
    base["resource_profile"]["profile_id"] = "custom"
    base["resource_profile"]["step_limit"] = 256
    rows = []
    operator_hash = digest(BINARY)
    cases = [(seed, enabled, mode, trial, False) for seed in [7, 19, 42]
             for enabled, mode, trial in [(True, "respecting", 0), (True, "respecting", 1),
                                          (False, "respecting", 0), (True, "oblivious", 0)]]
    cases.append((7, True, "respecting", 0, True))
    if args.provisional:
        cases = cases[:1]
    for seed, enabled, mode, trial, goal_terminates in cases:
        case = batch / f"s{seed}-{'options' if enabled else 'primitive'}-{mode}-r{trial}-goal{int(goal_terminates)}"
        case.mkdir()
        manifest = json.loads(json.dumps(base))
        spec = {"scenario_id": "feature-attainment-chain", "version": "1.0", "seed": seed,
                "horizon": 32, "action_count": 2, "observation_width": 1, "size": 5,
                "reward_state": 4, "goal_terminates": goal_terminates}
        manifest["environment"] = {"component_id": "bonsai-feature-chain", "version": "1.0.0",
                                   "entrypoint": [str(PYTHON), "-I", "-B", str(ROOT / "scripts/adapter_entrypoint.py"),
                                                  "chain_adapter"], "config": spec}
        manifest["scenario"] = {"scenario_id": spec["scenario_id"], "version": spec["version"],
                                "family": "feature-attainment-chain", "variant": "online-option-diagnostic",
                                "reward_unit_id": "scenario_reward", "config": spec}
        config = {"action_count": 2, "observation_width": 1, "seed": seed, "options_enabled": enabled,
                  "reward_mode": mode, "retained_state_limit_bytes": 1024 * 1024,
                  "serialized_state_limit_bytes": 256 * 1024}
        manifest["adapter"] = {
            "component_id": "bonsai-online-options", "version": "1.0.0", "config": config,
            "entrypoint": [str(PYTHON), "-I", "-B", str(ROOT / "scripts/adapter_entrypoint.py"), "option_adapter"],
            "accounting_contract": {"schema": "bonsai.online-accounting/v2",
                                    "feedback_signal_type": "bonsai.agent.causal-transition/v1",
                                    "parameter_update_schema": "bonsai.online-update/v2",
                                    "work_per_step": [{"work_class": key, "amount": value} for key, value in
                                                      [("acting", 6), ("learning", 1),
                                                       ("feature_generation", 172), ("option_learning", 72)]],
                                    "maximum_parameter_touches_per_update": 1024,
                                    "retained_state_limit_bytes": 1024 * 1024,
                                    "serialized_state_limit_bytes": 256 * 1024}}
        manifest["seeds"] = [{"seed_id": "environment", "value": str(seed)}]
        manifest["manifest_id"], manifest["run_id"] = str(uuid.uuid4()), str(uuid.uuid4())
        path = case / "manifest.json"
        path.write_text(json.dumps(manifest, indent=2) + "\n")
        started = time.monotonic_ns()
        run = subprocess.run([str(BINARY), "run", "--manifest", str(path), "--output", str(case / "run"),
                              "--authority", str(authority)], cwd=ROOT, capture_output=True, timeout=150)
        (case / "operator.stdout.txt").write_bytes(run.stdout)
        (case / "operator.stderr.txt").write_bytes(run.stderr)
        row = {"seed": seed, "horizon": 32, "steps": 256, "options_enabled": enabled, "reward_mode": mode,
               "trial": trial, "goal_terminates": goal_terminates, "case": case.relative_to(ROOT).as_posix(),
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
        assert digest(BINARY) == operator_hash
    if not args.provisional:
        assert source_snapshot() == source, "source changed during evidence run"
        assert subprocess.check_output(["git", "diff", "HEAD", "--binary"], cwd=ROOT,
                                       stderr=subprocess.DEVNULL, timeout=120) == patch
    identity = {"revision": revision, "tracked_patch_sha256": hashlib.sha256(patch).hexdigest(),
                "source_files": source, "executables": {BINARY.relative_to(ROOT).as_posix(): operator_hash},
                "python_executable_sha256": digest(PYTHON),
                "host": {"system": platform.system(), "release": platform.release(),
                         "machine": platform.machine(), "wsl": "microsoft" in platform.release().lower(),
                         "physical_acceptance": False}}
    (batch / "source-identity.json").write_text(json.dumps(identity, indent=2) + "\n")
    result = {"schema": "bonsai.online-option-diagnostic/v1", "provisional": args.provisional,
              "operator_sha256": operator_hash, "runs": rows, "resource_profile": base["resource_profile"],
              "scientific_quality_claim": False, "physical_host_acceptance": False}
    (batch / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"batch": batch.relative_to(ROOT).as_posix(), "runs": len(rows)}), flush=True)


if __name__ == "__main__":
    main()
