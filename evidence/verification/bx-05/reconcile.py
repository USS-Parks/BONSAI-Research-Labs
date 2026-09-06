"""Independently reconcile a completed runner bundle; no C0/C1 adjudication."""

import hashlib
import importlib
import json
import struct
import sys
import uuid
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path[:0] = [str(ROOT / "target/bx05-audit-proto"), str(ROOT / "python/bonsai-reference/src")]
event_wire = importlib.import_module("bonsai.event.v1.envelope_pb2")
adapter_wire = importlib.import_module("bonsai.adapter.v1.adapter_pb2")


def digest(data):
    return hashlib.sha256(data).digest()


def read_events(path):
    data = path.read_bytes()
    assert data[:8] == b"BNSSEG01" and digest(data[:28]) == data[28:60]
    footer = data[-88:]
    assert footer[:8] == b"BNSEND01" and digest(footer[:56]) == footer[56:]
    assert digest(data[:-88]) == footer[24:56]
    assert data[12:20] == footer[8:16]
    offset = 60
    events = []
    while offset < len(data) - 88:
        assert data[offset:offset + 8] == b"BNSFRM01"
        length = struct.unpack_from("<I", data, offset + 8)[0]
        assert 0 < length <= struct.unpack_from("<I", data, 20)[0]
        payload = data[offset + 12:offset + 12 + length]
        assert digest(payload) == data[offset + 12 + length:offset + 44 + length]
        event = event_wire.EventEnvelope.FromString(payload)
        assert event.source_sequence == len(events)
        assert event.availability == event_wire.AVAILABILITY_MEASURED
        assert event.payload_schema_epoch == 1 and event.payload_schema_minor == 0
        assert digest(event.payload) == event.payload_sha256
        assert list(event.causal_parent_event_ids) == ([events[-1].event_id] if events else [])
        if events:
            assert event.monotonic_time_ns >= events[-1].monotonic_time_ns
        events.append(event)
        offset += 44 + length
    assert offset == len(data) - 88 and len(events) == struct.unpack_from("<Q", footer, 16)[0]
    return events


def reconcile(output):
    observer = output / "observer"
    status = json.loads((observer / "run-status.json").read_text())
    manifest = json.loads((observer / "experiment-manifest.json").read_text())
    report = json.loads((observer / "reports/report.json").read_text())
    assert status["status"] == report["behavior"]["status"] == "COMPLETE"
    profile = manifest["resource_profile"]
    expected = profile["step_limit"]
    events = read_events(observer / "telemetry/segment-00000000000000000000.bseg")
    assert len(events) == status["events"] == report["behavior"]["events"]
    assert all(event.run_id == uuid.UUID(manifest["run_id"]).bytes for event in events)
    assert len({event.event_id for event in events}) == len(events)
    decoded = [(event.event_type, json.loads(event.payload)) for event in events]
    actions = [payload for kind, payload in decoded if kind == "run.action"]
    rewards = [payload for kind, payload in decoded if kind == "run.reward"]
    resources = [payload for kind, payload in decoded if kind == "run.resource" and payload.get("phase") == "step"]
    for population in [actions, rewards, resources]:
        assert [item["total_step"] for item in population] == list(range(expected))
    for action, reward in zip(actions, rewards, strict=True):
        transition = adapter_wire.CausalTransition.FromString(bytes.fromhex(reward["transition_hex"]))
        assert transition.action == action["action"] and transition.reward == reward["reward"]
        assert action["episode"] == reward["episode"] and action["step"] == reward["step"]
        observation = adapter_wire.CausalObservation.FromString(bytes.fromhex(action["observation_hex"]))
        assert action["action"] in observation.allowed_actions
    work = [payload for kind, payload in decoded if kind == "run.work"]
    counts = Counter((item["total_step"], item["phase"], item["work_class"]) for item in work)
    assert counts == Counter({(step, phase, work_class): 1 for step in range(expected)
                              for phase in ["admission", "measured_charge"] for work_class in ["acting", "learning"]})
    measured = [item for item in work if item["phase"] == "measured_charge"]
    for item in measured:
        accounting = adapter_wire.PrimitiveAccounting.FromString(bytes.fromhex(item["accounting_hex"]))
        steps = item["total_step"] + 1
        assert accounting.environment_steps == accounting.updates == steps
        assert accounting.parameter_touches == 2 * steps and accounting.replay_items_retained == 0
        assert accounting.work_items == (manifest["adapter"]["config"]["action_count"] + 1) * steps
    assert max(item["cpu_time_ns"] for item in resources) <= profile["per_step_cpu_time_limit_ns"]
    assert max(item["agent_rss_bytes"] for item in resources) <= profile["agent_rss_limit_bytes"]
    assert max(item["agent_storage_bytes"] for item in resources) <= profile["agent_storage_limit_bytes"]
    cleanup = [payload for kind, payload in decoded if kind == "run.resource" and payload.get("phase") == "cleanup"]
    assert {item["adapter"] for item in cleanup} == {"agent", "environment"}
    assert all(not item["usage"]["populated"] and not item["usage"]["member_pids"] for item in cleanup)
    pending = {}
    action_latencies = []
    for kind, item in decoded:
        if kind != "run.protocol":
            continue
        assert item["phase"] != "exchange_failed"
        frame = adapter_wire.AdapterFrame.FromString(bytes.fromhex(item["frame_hex"]))
        key = (item["adapter"], frame.sequence)
        if item["phase"] == "request_attempted":
            assert item["sent_complete"] and key not in pending
            pending[key] = (item["local_timeout_ns"], frame.WhichOneof("message"))
        else:
            deadline, request_kind = pending.pop(key)
            assert item["latency_ns"] <= deadline
            if item["adapter"] == "agent" and request_kind == "step":
                action_latencies.append(item["latency_ns"])
    assert not pending and len(action_latencies) == expected
    assert max(action_latencies) <= profile["action_deadline_ns"]
    reward_sum = sum(item["reward"] for item in rewards)
    estimate = json.loads((observer / "metric-estimate.json").read_text())
    assert reward_sum == estimate["result"]["value"] == report["behavior"]["reward_sum"]
    assert estimate["population"]["observed_count"] == expected and estimate["missingness"]["missing_count"] == 0
    assert status["execution_ns"] <= profile["wall_time_limit_ns"]
    index_path = observer / "artifact-index.json"
    assert digest(index_path.read_bytes()).hex() == status["artifact_index_sha256"]
    index = json.loads(index_path.read_text())
    for item in index["files"]:
        path = (observer / item["path"]).resolve()
        assert path.is_relative_to(observer.resolve())
        assert path.stat().st_size == item["bytes"] and digest(path.read_bytes()).hex() == item["sha256"]
    output_bytes = sum(path.stat().st_size for path in observer.rglob("*") if path.is_file())
    assert output_bytes <= profile["observer_output_limit_bytes"]
    return {"run_id": manifest["run_id"], "profile": profile["profile_id"], "events": len(events),
            "actions": len(actions), "rewards": len(rewards), "work_events": len(work),
            "resource_samples": len(resources),
            "reward_sum": reward_sum, "maximum_action_latency_ns": max(action_latencies),
            "maximum_step_cpu_ns": max(item["cpu_time_ns"] for item in resources), "observer_bytes": output_bytes,
            "indexed_artifacts": len(index["files"]), "verdict": "EXECUTION_RECONCILED", "C0_C1": "not_adjudicated"}


if __name__ == "__main__":
    print(json.dumps(reconcile(Path(sys.argv[1])), indent=2))
