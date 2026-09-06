"""Replay actual learner updates independently and prepare common conformance inputs."""
from __future__ import annotations

import hashlib
import importlib
import importlib.util
import json
import math
import subprocess
import sys
import time
from fractions import Fraction
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "target/bx10-audit-proto"))
wire = importlib.import_module("bonsai.adapter.v1.adapter_pb2")
importlib.import_module("bonsai.event.v1.envelope_pb2")
spec = importlib.util.spec_from_file_location("bx05_segment_reader", ROOT / "evidence/verification/bx-05/reconcile.py")
reader = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reader)


def close(actual, expected):
    if isinstance(expected, list):
        assert isinstance(actual, list) and len(actual) == len(expected)
        for left, right in zip(actual, expected, strict=True):
            close(left, right)
    else:
        assert math.isclose(actual, expected, rel_tol=1e-12, abs_tol=1e-12), (actual, expected)


def replay(case, agent):
    observer = case / "run/observer"
    manifest = json.loads((observer / "experiment-manifest.json").read_text())
    events = reader.read_events(observer / "telemetry/segment-00000000000000000000.bseg")
    decoded = [(event.event_type, json.loads(event.payload)) for event in events]
    actions = [p for k, p in decoded if k == "run.action"]
    rewards = [p for k, p in decoded if k == "run.reward"]
    measured = [p for k, p in decoded if k == "run.work" and p.get("phase") == "measured_charge"]
    profile = manifest["resource_profile"]
    steps = profile["step_limit"]
    assert len(actions) == len(rewards) == steps and len(measured) == 2 * steps
    count = manifest["adapter"]["config"]["action_count"]
    width = manifest["environment"]["config"]["observation_width"]
    weights = [[0.0] * (width + 1) for _ in range(count)]
    table = {}
    semantic = []
    changed = 0
    for i, (action, reward) in enumerate(zip(actions, rewards, strict=True)):
        assert action["total_step"] == reward["total_step"] == i
        observation = wire.CausalObservation.FromString(bytes.fromhex(action["observation_hex"]))
        values = tuple(observation.observation)
        chosen = action["action"]
        transition = wire.CausalTransition.FromString(bytes.fromhex(reward["transition_hex"]))
        assert transition.action == chosen and transition.reward == reward["reward"]
        assert chosen in observation.allowed_actions
        assert action["episode"] == reward["episode"] and action["step"] == reward["step"]
        pair = measured[2 * i:2 * i + 2]
        assert {p["work_class"] for p in pair} == {"acting", "learning"}
        assert all(p["total_step"] == i for p in pair)
        assert pair[0]["accounting_hex"] == pair[1]["accounting_hex"]
        accounting = wire.PrimitiveAccounting.FromString(bytes.fromhex(pair[0]["accounting_hex"]))
        assert accounting.environment_steps == accounting.updates == i + 1
        touches = 2 if agent == "tabular" else width + 1
        assert accounting.parameter_touches == touches * (i + 1)
        assert accounting.work_items == (count + 1) * (i + 1)
        assert accounting.replay_items_retained == 0
        update = json.loads(accounting.parameter_update)
        assert update["schema"] == "bonsai.parameter-update/v1"
        assert update["update"] == i + 1 and update["action"] == chosen and update["reward"] == reward["reward"]
        if agent == "tabular":
            unseen = [a for a in range(count) if (values, a) not in table]
            expected_action = unseen[0] if unseen else max(
                range(count), key=lambda a: Fraction(table[values, a][1], table[values, a][0]))
            assert chosen == expected_action
            before = table.get((values, chosen), [0, 0])
            after = [before[0] + 1, before[1] + reward["reward"]]
            assert update["rule"] == "tabular-sample-average" and update["observation"] == list(values)
            assert update["before"] == before and update["after"] == after
            table[values, chosen] = after
        else:
            features = [1.0] + [value / (1.0 + value) for value in values]
            predictions = [sum(w * x for w, x in zip(row, features, strict=True)) for row in weights]
            expected_action = i if i < count else max(range(count), key=predictions.__getitem__)
            assert chosen == expected_action, (i, chosen, expected_action)
            before = weights[chosen]
            factor = 0.25 * (reward["reward"] - predictions[chosen]) / sum(x * x for x in features)
            after = [w + factor * x for w, x in zip(before, features, strict=True)]
            assert update["rule"] == "linear-nlms"
            close(update["features"], features)
            close(update["prediction"], predictions[chosen])
            close(update["before"], before)
            close(update["after"], after)
            weights[chosen] = after
        changed += before != after
        semantic.append({"observation": list(values), "action": chosen, "reward": reward["reward"], "update": update})
    assert changed > 0
    resource = [p for k, p in decoded if k == "run.resource" and p.get("phase") == "step"]
    assert [p["total_step"] for p in resource] == list(range(steps))
    assert max(p["cpu_time_ns"] for p in resource) <= profile["per_step_cpu_time_limit_ns"]
    assert max(p["agent_rss_bytes"] for p in resource) <= profile["agent_rss_limit_bytes"]
    assert max(p["agent_storage_bytes"] for p in resource) <= profile["agent_storage_limit_bytes"]
    controls = next(p for k, p in decoded if k == "run.resource" and p.get("phase") == "controls_before_launch")
    assert controls["agent"]["cgroup_path"] != controls["environment"]["cgroup_path"]
    before_cleanup = next(p for k, p in decoded if k == "run.resource" and p.get("phase") == "before_cleanup")
    assert set(before_cleanup["agent"]["member_pids"]).isdisjoint(before_cleanup["environment"]["member_pids"])
    cleanup = [p for k, p in decoded if k == "run.resource" and p.get("phase") == "cleanup"]
    assert {p["adapter"] for p in cleanup} == {"agent", "environment"}
    assert all(not p["usage"]["populated"] and not p["usage"]["member_pids"] for p in cleanup)
    reaped = [p for k, p in decoded if k == "run.status" and p.get("phase") == "child_reaped"]
    assert len(reaped) == 2 and all(p["exit_code"] == 0 and not p["transport_failures"] for p in reaped)
    transcript = []
    pending = {}
    for kind, p in decoded:
        if kind != "run.protocol":
            continue
        assert p["phase"] != "exchange_failed"
        frame = wire.AdapterFrame.FromString(bytes.fromhex(p["frame_hex"]))
        key = (p["adapter"], frame.sequence)
        request = p["phase"] == "request_attempted"
        if request:
            assert p["sent_complete"] and key not in pending
            pending[key] = p["local_timeout_ns"]
        else:
            assert p["latency_ns"] <= pending.pop(key)
        if p["adapter"] == "agent":
            transcript.append({"peer": "supervisor" if request else "adapter", "hex": p["frame_hex"]})
    assert not pending
    audit = next(p["audit"] for k, p in decoded if k == "run.status" and p.get("phase") == "agent_launch_policy")
    lifecycle = [json.loads(line) for line in (observer / "lifecycle/lifecycle.jsonl").read_text().splitlines()]
    verdict = json.loads((case / "verifier.stdout.txt").read_text())
    operator = json.loads((case / "operator.stdout.txt").read_text())
    assert verdict["facts"]["receipt_sha256"] == operator["receipt_sha256"]
    assert verdict["facts"]["run_id"] == manifest["run_id"]
    assert verdict["facts"]["derived_track"] == "A"
    assert all(row["c0"] == row["c1"] == "pass" for row in verdict["claims"]["rows"])
    track = json.loads((observer / "track-declaration.json").read_text())
    # Completion comes from the independent verifier's source-pinned runtime
    # reconstruction above, never from the unverified manifest declaration.
    track["runtime_facts_complete"] = True
    certification = {
        "adapter_id": manifest["adapter"]["component_id"],
        "protocol_transcript": transcript,
        "capability_audit": audit,
        "observed_events": [event.SerializeToString().hex() for event in events],
        "lifecycle_records": lifecycle,
        "track_declaration": track,
    }
    semantic_hash = hashlib.sha256(json.dumps(semantic, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    return {
        "agent": agent, "steps": steps, "numeric_updates_changed": changed,
        "semantic_sha256": semantic_hash,
        "parameter_touches": touches * steps,
        "maximum_step_cpu_ns": max(p["cpu_time_ns"] for p in resource),
        "maximum_rss_bytes": max(p["agent_rss_bytes"] for p in resource),
        "resource_groups_separate": True,
    }, certification


def main():
    batch = ROOT / sys.argv[1]
    summary = json.loads((batch / "summary.json").read_text())
    analysis = batch / ("analysis-" + str(time.time_ns()))
    analysis.mkdir()
    results = []
    inputs = []
    identities = []
    for row in summary["runs"]:
        assert row["runner_exit"] == row["verifier_exit"] == 0
        case = ROOT / row["case"]
        manifest = json.loads((case / "manifest.json").read_text())
        assert manifest["resource_profile"] == summary["resource_profile"]
        assert manifest["environment"]["config"] == summary["scenario"]
        identity = json.loads((case / "run/observer/source-identity.json").read_text())
        assert identity["operator_executable_sha256"] == summary["operator_sha256"]
        identities.append(identity)
        result, certification = replay(case, row["agent"])
        result.update({"seed": row["seed"], "trial": row["trial"], "case": row["case"]})
        results.append(result)
        inputs.append(certification)
    if not summary["smoke"]:
        assert len(results) == 20
        assert {r["seed"] for r in results} == {7, 19, 42, 73, 91}
        assert all(identity == identities[0] for identity in identities)
        executable = ROOT / ("target/debug/examples/adapter_certification.exe" if sys.platform == "win32"
                             else "target/x86_64-unknown-linux-gnu/debug/examples/adapter_certification")
        for row, certification in zip(results, inputs, strict=True):
            pair = [r["semantic_sha256"] for r in results if r["agent"] == row["agent"] and r["seed"] == row["seed"]]
            assert len(pair) == 2 and len(set(pair)) == 1
            certification["determinism_probe_sha256"] = pair
            probe_path = batch / ("timeout-" + row["agent"]) / "timeout.json"
            certification["timeout_probe"] = json.loads(probe_path.read_text())
            case = ROOT / row["case"]
            output_case = analysis / case.name
            output_case.mkdir()
            input_path = output_case / "certification-input.json"
            with input_path.open("x") as output:
                json.dump(certification, output)
            subprocess.run([str(executable), "certify", str(input_path), str(output_case / "certification.json")],
                           cwd=ROOT, check=True, timeout=60)
    output = {"schema": "bonsai.paired-update-replay/v1", "smoke": summary["smoke"], "runs": results,
              "scientific_quality_claim": False}
    with (analysis / "replay.json").open("x") as stream:
        json.dump(output, stream, indent=2)
    print(json.dumps({"analysis": analysis.relative_to(ROOT).as_posix(), **output}, indent=2))


if __name__ == "__main__":
    main()
