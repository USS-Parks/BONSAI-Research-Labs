"""Replay online causal experience through live external feature admissions."""
from __future__ import annotations

import hashlib
import importlib
import json
import os
import platform
import subprocess
import sys
import time
import uuid
from dataclasses import asdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "python/bonsai-reference/src"))
sys.path.insert(0, str(ROOT / "target/bx12-audit-proto"))
discovery = importlib.import_module("bonsai_reference.feature_discovery")
admission = importlib.import_module("bonsai_reference.feature_admission")
scenario = importlib.import_module("bonsai_reference.scenario")
control = importlib.import_module("bonsai_reference.control")
wire = importlib.import_module("bonsai.artifact.v1.lineage_pb2")
availability = importlib.import_module("bonsai.event.v1.envelope_pb2")
binary_root = Path(os.environ.get("BONSAI_FEATURE_EXAMPLES", str(ROOT / "target/debug/examples")))
suffix = ".exe" if sys.platform == "win32" else ""
governor_exe = binary_root / ("feature_admission" + suffix)
lineage_exe = binary_root / ("feature_lineage" + suffix)


def lifecycle(proposal):
    provenance = wire.Provenance(
        producer_id=proposal["provenance"]["producer_id"],
        producer_version=proposal["provenance"]["producer_version"],
        source_event_ids=[uuid.UUID(value).bytes for value in proposal["provenance"]["source_event_ids"]],
        method_ids=proposal["provenance"]["method_ids"])
    common = {"artifact_id": uuid.UUID(proposal["artifact_id"]).bytes,
              "artifact_revision_id": uuid.UUID(proposal["artifact_revision_id"]).bytes,
              "lifecycle_sequence": proposal["lifecycle_sequence"]}
    content = {"representation_sha256": bytes.fromhex(proposal["representation_sha256"]),
               "provenance": provenance}
    if proposal["kind"] == "birth":
        event = wire.ArtifactLifecycleEvent(**common, birth=wire.ArtifactBirth(
            artifact_type=wire.ARTIFACT_TYPE_FEATURE, **content))
    else:
        event = wire.ArtifactLifecycleEvent(**common, revision=wire.ArtifactRevision(
            previous_revision_id=uuid.UUID(proposal["previous_revision_id"]).bytes, **content))
    common["lifecycle_sequence"] += 1
    cost = wire.ArtifactLifecycleEvent(**common, cost=wire.ArtifactCost(
        cost_entry_id=uuid.uuid5(uuid.UUID(proposal["artifact_revision_id"]), "serialized-bytes").bytes,
        counter_id="feature.representation.serialized_bytes", unit="bytes",
        amount=proposal["representation_serialized_bytes"], availability=availability.AVAILABILITY_MEASURED,
        provenance=provenance))
    return [item.SerializeToString(deterministic=True).hex() for item in [event, cost]]


def run(batch, seed, enabled, trial):
    case = batch / f"s{seed}-{'enabled' if enabled else 'disabled'}-r{trial}"
    case.mkdir()
    spec = scenario.ScenarioSpec("bx12-causal-diagnostic", "1.0", seed, 8, 2, 2, 8, (4,))
    environment = scenario.ScenarioSession(spec)
    actor = control.PrimitiveTabularControl(2)
    stage = discovery.OnlineFeatureStage(2, seed, enabled=enabled)
    governor = admission.FeatureGovernor(governor_exe)
    public, results, events = [], [], []
    episode = 0
    observed = environment.reset(seed)
    start = time.monotonic_ns()
    try:
        for step in range(128):
            action = actor.act(observed.observation)
            outcome = environment.step(observed.step, action)
            actor.observe(outcome.reward)
            source = str(uuid.uuid5(uuid.NAMESPACE_URL, f"bx12:{seed}:{step}"))
            row = {"event_id": source, "total_step": step, "episode": episode,
                   "observation": list(observed.observation), "action": action,
                   "reward": outcome.reward, "next": list(outcome.next.observation),
                   "terminated": outcome.terminated, "truncated": outcome.truncated}
            public.append(row)
            result = stage.observe(step, observed.observation, outcome.reward, source, governor)
            assert result["accepted"], result
            assert result["serialized_bytes"] == len(discovery.canonical(stage.online_snapshot()))
            assert result["allocated_bytes"] == sys.getsizeof(stage) + discovery.retained_bytes(vars(stage))
            results.append(result)
            for proposal in result["events"]:
                events.extend(lifecycle(proposal))
            if outcome.terminated or outcome.truncated:
                episode += 1
                actor.reset_episode()
                observed = environment.reset(seed + episode)
            else:
                observed = outcome.next
    finally:
        governor.close()
    assert governor.returncode == 0
    elapsed = time.monotonic_ns() - start
    snapshot = stage.online_snapshot()
    proposals = [event for result in results for event in result["events"]]
    if enabled:
        assert {"birth", "revision"} <= {event["kind"] for event in proposals}
        assert stage.snapshot()
    else:
        assert not proposals and not stage.snapshot()
    (case / "public-stream.json").write_text(json.dumps(public, indent=2) + "\n")
    (case / "admission-trace.json").write_text(json.dumps(results, indent=2) + "\n")
    (case / "feature-state.json").write_bytes(discovery.canonical(snapshot))
    (case / "lineage.json").write_text(json.dumps(events) + "\n")
    if enabled:
        checked = subprocess.run([str(lineage_exe), str(case / "lineage.json")],
                                 capture_output=True, check=True, timeout=30)
        (case / "lineage-verifier.stdout.txt").write_bytes(checked.stdout)
        (case / "lineage-verifier.stderr.txt").write_bytes(checked.stderr)
    result = {"case": case.relative_to(ROOT).as_posix(), "seed": seed, "enabled": enabled,
              "trial": trial, "steps": 128, "elapsed_ns": elapsed,
              "governor_pid": governor.pid, "governor_returncode": governor.returncode,
              "governor_reaped": governor.returncode == 0,
              "trajectory_sha256": hashlib.sha256(discovery.canonical(public)).hexdigest(),
              "state_sha256": hashlib.sha256(discovery.canonical(snapshot)).hexdigest(),
              "lineage_sha256": hashlib.sha256(discovery.canonical(events)).hexdigest(),
              "lineage_events": len(events), "births": sum(p["kind"] == "birth" for p in proposals),
              "revisions": sum(p["kind"] == "revision" for p in proposals),
              "maximum_retained_bytes": max(row["allocated_bytes"] for row in results),
              "maximum_serialized_bytes": max(row["serialized_bytes"] for row in results),
              "work_charged": sum(row["work"]["requested"] for row in results),
              "actor_accounting": asdict(actor.accounting)}
    (case / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


def source_snapshot():
    paths = []
    for directory in ["crates", "proto", "schemas", "scripts", "python/bonsai-reference/src",
                      "python/bonsai-reference/tests", "evidence/verification/bx-12"]:
        paths.extend(path for path in (ROOT / directory).rglob("*")
                     if path.is_file() and "__pycache__" not in path.parts
                     and path.suffix in {".rs", ".py", ".pyi", ".proto", ".json", ".toml", ".lock"})
    paths.extend(ROOT / name for name in ["Cargo.toml", "Cargo.lock", "pyproject.toml", "uv.lock",
                                         "rust-toolchain.toml", ".github/workflows/ci.yml", ".gitattributes"])
    assert len(paths) <= 10000
    assert all(path.stat().st_size <= 16 * 1024 * 1024 for path in paths)
    return {path.relative_to(ROOT).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(paths)}


def main():
    assert not sys.argv[1:]
    batch = ROOT / "target" / ("bx12-live-" + str(time.time_ns()))
    batch.mkdir()
    source = source_snapshot()
    patch = subprocess.check_output(["git", "diff", "HEAD", "--binary"], stderr=subprocess.DEVNULL, timeout=120)
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True, timeout=120).strip()
    hashes = {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
              for path in [governor_exe, lineage_exe]}
    rows = []
    for seed in [7, 19, 42]:
        for enabled, trial in [(True, 0), (True, 1), (False, 0)]:
            row = run(batch, seed, enabled, trial)
            rows.append(row)
            print(json.dumps(row), flush=True)
        pair = rows[-3:]
        assert len({row["trajectory_sha256"] for row in pair}) == 1
        assert pair[0]["state_sha256"] == pair[1]["state_sha256"]
        assert pair[0]["lineage_sha256"] == pair[1]["lineage_sha256"]
    assert hashes == {path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest() for path in hashes}
    assert source_snapshot() == source
    assert subprocess.check_output(["git", "diff", "HEAD", "--binary"], stderr=subprocess.DEVNULL, timeout=120) == patch
    identity = {"revision": revision, "tracked_patch_sha256": hashlib.sha256(patch).hexdigest(),
                "source_files": source, "executables": hashes, "python_version": sys.version,
                "host": {"system": platform.system(), "release": platform.release(),
                         "machine": platform.machine(), "hosted_ci": os.getenv("GITHUB_ACTIONS") == "true",
                         "wsl": "microsoft" in platform.release().lower(), "physical_acceptance": False},
                "python_executable_sha256": hashlib.sha256(Path(sys.executable).read_bytes()).hexdigest()}
    (batch / "source-identity.json").write_text(json.dumps(identity, indent=2) + "\n")
    summary = {"schema": "bonsai.online-feature-diagnostic/v1", "runs": rows, "executables": hashes,
               "source_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True, timeout=120).strip(),
               "scientific_quality_claim": False, "physical_host_acceptance": False}
    (batch / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    subprocess.run([sys.executable, "-B", str(ROOT / "evidence/verification/bx-12/replay.py"),
                    str(batch.relative_to(ROOT))], cwd=ROOT, check=True, timeout=60)
    print(json.dumps({"batch": batch.relative_to(ROOT).as_posix(), "runs": len(rows)}))


if __name__ == "__main__":
    main()
