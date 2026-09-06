"""Independently reconstruct BX-12 feature evidence from the public experience stream."""
from __future__ import annotations

import copy
import hashlib
import importlib
import json
import sys
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "target/bx12-audit-proto"))
wire = importlib.import_module("bonsai.artifact.v1.lineage_pb2")
availability = importlib.import_module("bonsai.event.v1.envelope_pb2")


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def digest(value):
    return hashlib.sha256(encoded(value)).hexdigest()


def read(path):
    assert path.is_file() and path.stat().st_size <= 16 * 1024 * 1024, path
    return json.loads(path.read_bytes())


def check(public, trace, state, lineage, summary):
    """Do not import or call OnlineFeatureStage or its diagnostic generator."""
    seed, enabled = summary["seed"], summary["enabled"]
    assert seed in {7, 19, 42} and type(enabled) is bool
    assert len(public) == len(trace) == summary["steps"] == 128
    assert summary["governor_pid"] > 0 and summary["governor_returncode"] == 0
    assert summary["governor_reaped"] is True
    stats, features, versions = {}, {}, {}
    expected = {"schema": "bonsai.online-feature-state/v1", "width": 2, "seed": seed,
                "enabled": enabled, "max_statistics": 64, "max_features": 16,
                "next_step": 0, "feature_work": 0, "features": [], "statistics": [],
                "versions": versions, "replay_items_retained": 0}
    cost_index = 0
    births = revisions = high_water = storage_sequence = 0
    tariff = 400 if enabled else 1
    for step, (experience, decision) in enumerate(zip(public, trace, strict=True)):
        assert experience["total_step"] == step
        source = str(uuid.uuid5(uuid.NAMESPACE_URL, f"bx12:{seed}:{step}"))
        assert experience["event_id"] == source
        assert decision["accepted"] is True and decision["reason_code"] == "FEATURE_STATE_ADMITTED"
        assert decision["state_before"] == digest(expected)
        work = decision["work"]
        assert work == {"authority": "agent", "consumed_after": tariff * (step + 1),
                        "consumed_before": tariff * step, "outcome": "admit", "purpose": "ordinary",
                        "reason_code": "WORK_CLASS_RESERVATION_AVAILABLE", "request_id": f"feature-work-{step}",
                        "requested": tariff, "reservation": 1000000, "sequence": step,
                        "work_class": "feature_generation"}
        eligible = []
        if enabled:
            for coordinate, value in enumerate(experience["observation"]):
                key = (coordinate, value)
                if key not in stats:
                    if len(stats) == 64:
                        continue
                    stats[key] = {"count": 0, "reward_sum": 0, "first_step": step,
                                  "last_step": step, "first_event_id": source, "last_event_id": source}
                exposure = stats[key]
                exposure["count"] += 1
                exposure["reward_sum"] += experience["reward"]
                exposure["last_step"], exposure["last_event_id"] = step, source
                if exposure["reward_sum"] > 0 and (exposure["count"] == 2 or exposure["count"] % 4 == 0):
                    eligible.append(key)
        ordered = sorted(eligible, key=lambda key: hashlib.sha256(encoded([seed, step, key])).digest())
        candidate = None
        for coordinate, value in ordered:
            identity = str(uuid.uuid5(uuid.NAMESPACE_URL, f"bonsai-feature:{seed}:{coordinate}:{value}"))
            if identity in features or len(features) < 16:
                candidate = (identity, coordinate, value)
                break
        proposals = decision["events"]
        assert len(proposals) == int(candidate is not None)
        if candidate is not None:
            identity, coordinate, value = candidate
            exposure = stats[(coordinate, value)]
            old = versions.get(identity)
            version = 1 if old is None else old[0] + 1
            revision = str(uuid.uuid5(uuid.UUID(identity), str(version)))
            representation = [0, coordinate, value, exposure["reward_sum"], exposure["count"]]
            provenance = {"producer_id": "bonsai-online-features", "producer_version": "1.0.0",
                          "source_event_ids": sorted({exposure["first_event_id"], exposure["last_event_id"]}),
                          "method_ids": ["observed-equality-positive-support-v1"]}
            proposal = {"artifact_id": identity, "artifact_revision_id": revision,
                        "lifecycle_sequence": 2 * version - 1, "kind": "birth" if old is None else "revision",
                        "previous_revision_id": None if old is None else old[1],
                        "representation": representation, "representation_sha256": digest(representation),
                        "parents": [], "exposure": dict(exposure), "provenance": provenance, "utility": None,
                        "parameter_count": 2, "representation_serialized_bytes": len(encoded(representation))}
            assert proposals == [proposal]
            if old is None:
                features[identity] = {"feature_id": identity, "representation": representation,
                                      "birth_step": step, "retirement_step": None,
                                      "bytes": len(encoded(representation)), "work": 2,
                                      "consumers": 0, "utility": None, "parents": []}
                expected["feature_work"] += 2
                births += 1
            else:
                features[identity].update(representation=representation, bytes=len(encoded(representation)))
                features[identity]["work"] += 1
                expected["feature_work"] += 1
                revisions += 1
            versions[identity] = [version, revision, 2 * version]
            event, cost = [wire.ArtifactLifecycleEvent.FromString(bytes.fromhex(raw))
                           for raw in lineage[cost_index:cost_index + 2]]
            cost_index += 2
            for message, sequence in [(event, 2 * version - 1), (cost, 2 * version)]:
                assert message.artifact_id == uuid.UUID(identity).bytes
                assert message.artifact_revision_id == uuid.UUID(revision).bytes
                assert message.lifecycle_sequence == sequence
            assert event.WhichOneof("detail") == proposal["kind"]
            body = getattr(event, proposal["kind"])
            assert body.representation_sha256.hex() == digest(representation)
            assert list(body.parents) == []
            if old is not None:
                assert body.previous_revision_id == uuid.UUID(old[1]).bytes
            else:
                assert body.artifact_type == wire.ARTIFACT_TYPE_FEATURE
            assert cost.WhichOneof("detail") == "cost"
            assert cost.cost.cost_entry_id == uuid.uuid5(uuid.UUID(revision), "serialized-bytes").bytes
            assert cost.cost.counter_id == "feature.representation.serialized_bytes"
            assert cost.cost.unit == "bytes" and cost.cost.amount == len(encoded(representation))
            assert cost.cost.availability == availability.AVAILABILITY_MEASURED
            for origin in [body.provenance, cost.cost.provenance]:
                assert origin.producer_id == provenance["producer_id"]
                assert origin.producer_version == provenance["producer_version"]
                assert list(origin.source_event_ids) == [
                    uuid.UUID(item).bytes for item in provenance["source_event_ids"]]
                assert list(origin.method_ids) == provenance["method_ids"]
        expected["next_step"] = step + 1
        expected["features"] = [features[key] for key in sorted(features)]
        expected["statistics"] = [{"coordinate": key[0], "value": key[1], **stats[key]} for key in sorted(stats)]
        size = len(encoded(expected))
        assert decision["state_after"] == digest(expected)
        assert decision["serialized_bytes"] == size
        assert decision["feature_count"] == len(features) and decision["statistics_count"] == len(stats)
        allocation = decision["allocation"]
        assert allocation["request_id"] == f"feature-state-{step}" and allocation["outcome"] == "admit"
        assert 0 < decision["allocated_bytes"] == allocation["allocated_bytes"] <= allocation["memory_limit"] == 262144
        assert allocation["serialized_bytes"] == size <= 65536
        if step == 0 or size > high_water:
            storage = allocation["storage"]
            assert storage == {"byte_delta": size - high_water, "bytes_after": size,
                               "bytes_before": high_water, "classified_kind": "bounded_algorithm_state",
                               "declared_kind": "bounded_algorithm_state", "file_delta": int(step == 0),
                               "files_after": 1, "files_before": int(step != 0),
                               "logical_path": "feature-state.json", "outcome": "admit",
                               "reason_code": "BOUNDED_ALGORITHM_STATE_ALLOWED",
                               "request_id": f"feature-state-{step}", "sequence": storage_sequence,
                               "track_fact_replay_capacity": 0}
            assert allocation["reason_code"] == storage["reason_code"]
            high_water = size
            storage_sequence += 1
        else:
            assert allocation["reason_code"] == "FEATURE_EXISTING_STATE_RESERVATION"
            assert allocation["storage_usage"] == {"bytes_consumed": high_water,
                                                   "classified_transition_capacity": 0, "files_consumed": 1,
                                                   "max_bytes": 65536, "max_files": 1}
    assert state == expected
    assert cost_index == len(lineage) == summary["lineage_events"]
    assert (births, revisions) == (summary["births"], summary["revisions"])
    assert bool(births and revisions) == enabled
    assert summary["work_charged"] == tariff * 128
    assert summary["maximum_retained_bytes"] == max(row["allocated_bytes"] for row in trace)
    assert summary["maximum_serialized_bytes"] == max(row["serialized_bytes"] for row in trace)
    assert summary["trajectory_sha256"] == digest(public)
    assert summary["state_sha256"] == digest(state)
    assert summary["lineage_sha256"] == digest(lineage)
    return {"seed": seed, "enabled": enabled, "steps_reconstructed": 128,
            "births": births, "revisions": revisions, "lineage_events": len(lineage)}


def tampering_checks(items):
    rejected = []
    for mutation in ["reward", "missing_step", "work", "representation", "utility", "state", "lineage", "track"]:
        public, trace, state, lineage, summary = copy.deepcopy(items)
        proposed = next(row for row in trace if row["events"])
        if mutation == "reward":
            public[0]["reward"] += 1
            summary["trajectory_sha256"] = digest(public)
        elif mutation == "missing_step":
            public.pop()
        elif mutation == "work":
            trace[0]["work"]["requested"] = 0
        elif mutation == "track":
            trace[0]["allocation"]["storage"]["classified_kind"] = "transition_replay"
        elif mutation == "representation":
            proposed["events"][0]["representation"][3] += 1
        elif mutation == "utility":
            proposed["events"][0]["utility"] = 999
        elif mutation == "state":
            state["statistics"][0]["count"] += 1
            summary["state_sha256"] = digest(state)
        else:
            lineage.pop()
            summary["lineage_sha256"] = digest(lineage)
        try:
            check(public, trace, state, lineage, summary)
        except (AssertionError, ValueError, IndexError):
            rejected.append(mutation)
        else:
            raise AssertionError("altered evidence accepted: " + mutation)
    return rejected


def main():
    assert len(sys.argv) == 2
    batch = (ROOT / sys.argv[1]).resolve()
    assert batch.is_relative_to(ROOT / "target")
    summary = read(batch / "summary.json")
    assert len(summary["runs"]) == 9
    results, reference = [], None
    cases = set()
    for row in summary["runs"]:
        name = f"s{row['seed']}-{'enabled' if row['enabled'] else 'disabled'}-r{row['trial']}"
        case = batch / name
        assert (ROOT / row["case"]).resolve() == case and name not in cases
        cases.add(name)
        items = [read(case / name) for name in ["public-stream.json", "admission-trace.json",
                                                "feature-state.json", "lineage.json", "summary.json"]]
        assert items[-1] == row
        results.append(check(*items))
        if reference is None and row["enabled"]:
            reference = items
    assert cases == {f"s{seed}-{kind}-r{trial}" for seed in [7, 19, 42]
                     for kind, trial in [("enabled", 0), ("enabled", 1), ("disabled", 0)]}
    for seed in [7, 19, 42]:
        rows = [row for row in summary["runs"] if row["seed"] == seed]
        assert len({row["trajectory_sha256"] for row in rows}) == 1
        enabled = [row for row in rows if row["enabled"]]
        assert enabled[0]["state_sha256"] == enabled[1]["state_sha256"]
        assert enabled[0]["lineage_sha256"] == enabled[1]["lineage_sha256"]
    report = {"schema": "bonsai.online-feature-reconstruction/v1", "runs": results,
              "reconstructed_steps": sum(row["steps_reconstructed"] for row in results),
              "tampering_rejected": tampering_checks(reference), "result": "pass",
              "allocated_bytes_scope": "recorded retained-object measurements; not OS RSS or peak allocation"}
    (batch / "reconstruction.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
