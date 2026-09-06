"""Independently reconstruct BX-13 governed option-learning evidence."""

from __future__ import annotations

import copy
import hashlib
import importlib
import importlib.util
import json
import sys
from collections.abc import Callable
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "target/bx13-audit-proto"))
wire = importlib.import_module("bonsai.adapter.v1.adapter_pb2")
importlib.import_module("bonsai.event.v1.envelope_pb2")
reader_spec = importlib.util.spec_from_file_location(
    "bx13_bounded_segment_reader", ROOT / "evidence/verification/bx-05/reconcile.py"
)
assert reader_spec is not None and reader_spec.loader is not None
reader = importlib.util.module_from_spec(reader_spec)
reader_spec.loader.exec_module(reader)

from option_reconstruction import (  # noqa: E402
    MAX_PARAMETER_TOUCHES,
    MAX_RETAINED_BYTES,
    MAX_SERIALIZED_BYTES,
    MAX_UPDATE_BYTES,
    canonical,
    feedback_event_id,
    reconstruct_records,
)

WORK_CLASSES = ("acting", "learning", "feature_generation", "option_learning")
Package = tuple[
    dict[str, Any],
    list[dict[str, Any]],
    list[dict[str, Any]],
    list[dict[str, Any]],
    int,
]


def read_json(path: Path) -> Any:
    assert path.is_file() and path.stat().st_size <= 16 * 1024 * 1024, path
    return json.loads(path.read_bytes())


def sha256(path: Path) -> str:
    assert path.is_file() and path.stat().st_size <= 16 * 1024 * 1024, path
    return hashlib.sha256(path.read_bytes()).hexdigest()


def lower_hash(value: object) -> bool:
    return (
        isinstance(value, str)
        and len(value) == 64
        and all(character in "0123456789abcdef" for character in value)
    )


def expected_tariffs(actions: int, width: int) -> dict[str, int]:
    return {
        "acting": actions + 4,
        "learning": 1,
        "feature_generation": 168 + 4 * width,
        "option_learning": 16 + 4 * (8 + 2 * actions + 2 * width),
    }


def check_manifest(case: Path, row: dict[str, Any]) -> tuple[dict[str, Any], dict[str, Any]]:
    manifest = read_json(case / "manifest.json")
    observer = case / "run/observer"
    retained = read_json(observer / "experiment-manifest.json")
    assert retained == manifest
    agent, environment = manifest["adapter"], manifest["environment"]
    config, environment_config = agent["config"], environment["config"]
    assert read_json(observer / "agent-configuration.json") == config
    assert read_json(observer / "environment-configuration.json") == environment_config
    assert agent["component_id"] == "bonsai-online-options" and agent["version"] == "1.0.0"
    assert environment["component_id"] == "bonsai-feature-chain"
    assert environment["version"] == "1.0.0"
    assert set(config) == {
        "action_count",
        "observation_width",
        "seed",
        "options_enabled",
        "reward_mode",
        "retained_state_limit_bytes",
        "serialized_state_limit_bytes",
    }
    assert config == {
        "action_count": 2,
        "observation_width": 1,
        "seed": row["seed"],
        "options_enabled": row["options_enabled"],
        "reward_mode": row["reward_mode"],
        "retained_state_limit_bytes": MAX_RETAINED_BYTES,
        "serialized_state_limit_bytes": MAX_SERIALIZED_BYTES,
    }
    assert set(environment_config) == {
        "scenario_id",
        "version",
        "seed",
        "horizon",
        "action_count",
        "observation_width",
        "size",
        "reward_state",
        "goal_terminates",
    }
    assert environment_config == {
        "scenario_id": "feature-attainment-chain",
        "version": "1.0",
        "seed": row["seed"],
        "horizon": row["horizon"],
        "action_count": 2,
        "observation_width": 1,
        "size": 5,
        "reward_state": 4,
        "goal_terminates": row["goal_terminates"],
    }
    tariffs = expected_tariffs(2, 1)
    assert agent["accounting_contract"] == {
        "schema": "bonsai.online-accounting/v2",
        "feedback_signal_type": "bonsai.agent.causal-transition/v1",
        "parameter_update_schema": "bonsai.online-update/v2",
        "work_per_step": [
            {"work_class": key, "amount": value} for key, value in tariffs.items()
        ],
        "maximum_parameter_touches_per_update": MAX_PARAMETER_TOUCHES,
        "retained_state_limit_bytes": MAX_RETAINED_BYTES,
        "serialized_state_limit_bytes": MAX_SERIALIZED_BYTES,
    }
    assert manifest["seeds"] == [{"seed_id": "environment", "value": str(row["seed"])}]
    track = manifest["track"]
    assert track["declared_track"] == "A" and track["batch_size"] == 1
    assert track["replay"] == {"enabled": False, "source": "none", "capacity_transitions": 0}
    assert track["offline_updates"] is False
    assert track["observer_data_access"] is False
    assert track["privileged_inputs"] is False
    assert track["human_labels"] is False
    assert manifest["scenario"]["config"] == environment_config
    assert manifest["resource_profile"]["step_limit"] == row["steps"]
    assert row["runner_exit"] == row["verifier_exit"] == 0
    assert lower_hash(row["receipt_sha256"])
    receipt = observer / "run-receipt.json"
    assert sha256(receipt) == row["receipt_sha256"]
    assert read_json(receipt)["run_id"] == manifest["run_id"]
    declaration = read_json(observer / "track-declaration.json")
    assert declaration["declared_track"] == "A"
    assert declaration["replay_capacity_transitions"] == 0
    assert declaration["offline_updates"] is False
    assert declaration["observer_data_access"] is False
    return manifest, config


def check_identities(
    case: Path, manifest: dict[str, Any], batch_identity: dict[str, Any]
) -> dict[str, Any]:
    observer = case / "run/observer"
    components = read_json(observer / "component-identity.json")
    source = read_json(observer / "source-identity.json")
    assert components["agent"]["component_id"] == "bonsai-online-options"
    assert components["environment"]["component_id"] == "bonsai-feature-chain"
    assert source["schema"] == "bonsai.run-source-identity/v1"
    assert source["revision"] == manifest["source"]["revision"] == batch_identity["revision"]
    assert source["dirty"] == manifest["source"]["dirty"] is True
    assert source["tracked_patch_sha256"] == manifest["source"]["dirty_patch_sha256"]
    assert source["tracked_patch_sha256"] == batch_identity["tracked_patch_sha256"]
    required_runtime_sources = {
        "python/bonsai-reference/src/bonsai_reference/feature_discovery.py",
        "python/bonsai-reference/src/bonsai_reference/option_learning.py",
        "python/bonsai-reference/src/bonsai_reference/option_adapter.py",
        "python/bonsai-reference/src/bonsai_reference/chain_adapter.py",
    }
    assert required_runtime_sources <= set(source["source_files"])
    common = set(source["source_files"]) & set(batch_identity["source_files"])
    assert required_runtime_sources <= common
    assert all(source["source_files"][key] == batch_identity["source_files"][key] for key in common)
    assert all(lower_hash(value) for value in source["source_files"].values())
    assert source["operator_executable_sha256"] in batch_identity["executables"].values()
    for role, component in (("agent", manifest["adapter"]), ("environment", manifest["environment"])):
        files = components[role]["entrypoint_files"]
        assert set(files) == {component["entrypoint"][0], component["entrypoint"][3]}
        assert files[component["entrypoint"][0]] == batch_identity["python_executable_sha256"]
        assert files[component["entrypoint"][3]] == source["source_files"]["scripts/adapter_entrypoint.py"]
    return {
        "revision": source["revision"],
        "tracked_patch_sha256": source["tracked_patch_sha256"],
        "source_files": len(source["source_files"]),
        "operator_sha256": source["operator_executable_sha256"],
    }


def check_resources(decoded: list[tuple[str, dict[str, Any]]], profile: dict[str, Any]) -> dict[str, Any]:
    controls = [
        payload
        for kind, payload in decoded
        if kind == "run.resource" and payload.get("phase") == "controls_before_launch"
    ]
    assert len(controls) == 1
    roots: list[str] = []
    for role, memory_limit in (
        ("agent", profile["agent_rss_limit_bytes"]),
        ("environment", 134_217_728),
    ):
        value = controls[0][role]
        assert value["memory_max"] == value["limits"]["memory_max_bytes"] == memory_limit
        assert value["memory_swap_max"] == 0 and value["memory_oom_group"] == 1
        assert value["pids_max"] == value["limits"]["pids_max"] == 32
        assert value["cpu_max"] == "100000 100000"
        path = value["cgroup_path"]
        assert path.startswith("/sys/fs/cgroup/") and "/../" not in path
        roots.append(path.rsplit("/", 1)[0])
    assert roots[0] == roots[1]
    samples = [
        payload
        for kind, payload in decoded
        if kind == "run.resource" and payload.get("phase") == "step"
    ]
    assert [sample["total_step"] for sample in samples] == list(range(profile["step_limit"]))
    for sample in samples:
        usage = sample["usage"]
        assert 0 < sample["agent_rss_bytes"] <= profile["agent_rss_limit_bytes"]
        assert sample["agent_storage_bytes"] <= profile["agent_storage_limit_bytes"]
        assert sample["agent_storage_objects"] <= 4096
        assert sample["cpu_time_ns"] <= profile["per_step_cpu_time_limit_ns"]
        assert usage["populated"] is True and usage["member_pids"]
        assert 0 < usage["pids_current"] <= 32
        assert usage["memory_current_bytes"] <= profile["agent_rss_limit_bytes"]
        assert usage["memory_oom_kills"] == usage["memory_max_events"] == 0
        assert usage["pids_max_events"] == 0
    cleanup = [
        payload
        for kind, payload in decoded
        if kind == "run.resource" and payload.get("phase") == "cleanup"
    ]
    assert {value["adapter"] for value in cleanup} == {"agent", "environment"}
    assert all(
        value["usage"]["populated"] is False
        and value["usage"]["member_pids"] == []
        and value["usage"]["pids_current"] == 0
        for value in cleanup
    )
    return {
        "cgroup_parent": roots[0],
        "samples": len(samples),
        "maximum_agent_rss_bytes": max(sample["agent_rss_bytes"] for sample in samples),
        "maximum_cgroup_memory_current_bytes": max(
            sample["usage"]["memory_current_bytes"] for sample in samples
        ),
        "maximum_step_cpu_ns": max(sample["cpu_time_ns"] for sample in samples),
        "maximum_agent_storage_bytes": max(sample["agent_storage_bytes"] for sample in samples),
        "children_reaped_and_cgroups_empty": True,
    }


def extract_records(
    events: list[Any], row: dict[str, Any]
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]], list[tuple[str, dict[str, Any]]]]:
    decoded = [(event.event_type, json.loads(event.payload)) for event in events]
    actions = [payload for kind, payload in decoded if kind == "run.action"]
    rewards = [payload for kind, payload in decoded if kind == "run.reward"]
    measured = [
        payload
        for kind, payload in decoded
        if kind == "run.work" and payload.get("phase") == "measured_charge"
    ]
    steps = row["steps"]
    assert [value["total_step"] for value in actions] == list(range(steps))
    assert [value["total_step"] for value in rewards] == list(range(steps))
    assert [(value["total_step"], value["work_class"]) for value in measured] == [
        (step, work_class) for step in range(steps) for work_class in WORK_CLASSES
    ]
    transitions: list[dict[str, Any]] = []
    audits: list[dict[str, Any]] = []
    accountings: list[dict[str, Any]] = []
    expected_observation: int | None = None
    prior_episode = -1
    prior_done = True
    for total, (action_event, reward_event) in enumerate(zip(actions, rewards, strict=True)):
        observation = wire.CausalObservation.FromString(bytes.fromhex(action_event["observation_hex"]))
        transition = wire.CausalTransition.FromString(bytes.fromhex(reward_event["transition_hex"]))
        assert transition.HasField("next")
        next_observation = transition.next
        episode, episode_step = action_event["episode"], action_event["step"]
        assert reward_event["episode"] == episode and reward_event["step"] == episode_step
        assert action_event["total_step"] == reward_event["total_step"] == total
        assert observation.step == episode_step and transition.step == episode_step
        assert transition.action == action_event["action"]
        assert transition.reward == reward_event["reward"]
        assert reward_event["terminated"] == transition.terminated
        assert reward_event["truncated"] == transition.truncated
        assert len(observation.observation) == len(next_observation.observation) == 1
        if prior_done:
            assert episode == prior_episode + 1 and episode_step == 0
            expected_observation = (row["seed"] + episode) % 4
        else:
            assert episode == prior_episode and expected_observation is not None
        before = observation.observation[0]
        assert before == expected_observation and 0 <= before < 5
        assert list(observation.allowed_actions) == [0, 1]
        after = (before + (1 if transition.action == 0 else -1)) % 5
        attained = after == 4
        expected_terminated = attained and row["goal_terminates"]
        expected_truncated = not expected_terminated and episode_step + 1 == row["horizon"]
        assert list(next_observation.observation) == [after]
        assert next_observation.step == episode_step + 1
        assert next_observation.stream_id == observation.stream_id
        assert transition.reward == (4 if attained else -1)
        assert transition.terminated == expected_terminated
        assert transition.truncated == expected_truncated
        done = transition.terminated or transition.truncated
        assert list(next_observation.allowed_actions) == ([] if done else [0, 1])
        transitions.append(
            {
                "total_step": total,
                "episode": episode,
                "episode_step": episode_step,
                "observation": [before],
                "action": transition.action,
                "reward": transition.reward,
                "next_observation": [after],
                "terminated": bool(transition.terminated),
                "truncated": bool(transition.truncated),
                "source_event_id": feedback_event_id(total),
            }
        )
        copies = measured[total * len(WORK_CLASSES) : (total + 1) * len(WORK_CLASSES)]
        accounting_hex = copies[0]["accounting_hex"]
        assert all(value["accounting_hex"] == accounting_hex for value in copies)
        accounting = wire.PrimitiveAccounting.FromString(bytes.fromhex(accounting_hex))
        audit = json.loads(accounting.parameter_update)
        assert canonical(audit) == accounting.parameter_update
        assert len(accounting.parameter_update) <= MAX_UPDATE_BYTES
        audits.append(audit)
        accountings.append(
            {
                "environment_steps": accounting.environment_steps,
                "updates": accounting.updates,
                "parameter_touches": accounting.parameter_touches,
                "work_items": accounting.work_items,
                "replay_items_retained": accounting.replay_items_retained,
            }
        )
        expected_observation = after
        prior_episode, prior_done = episode, done
    return transitions, audits, accountings, decoded


def reconstruct_case(
    case: Path,
    row: dict[str, Any],
    batch_identity: dict[str, Any],
) -> tuple[dict[str, Any], Package]:
    manifest, config = check_manifest(case, row)
    identity = check_identities(case, manifest, batch_identity)
    events = reader.read_events(case / "run/observer/telemetry/segment-00000000000000000000.bseg")
    transitions, audits, accountings, decoded = extract_records(events, row)
    state, result = reconstruct_records(config, transitions, audits, accountings, row["steps"])
    resources = check_resources(decoded, manifest["resource_profile"])
    allocated = [audit["allocated_bytes"] for audit in audits]
    serialized = [audit["serialized_bytes"] for audit in audits]
    assert all(0 < value <= MAX_RETAINED_BYTES for value in allocated)
    assert all(0 < value <= MAX_SERIALIZED_BYTES for value in serialized)
    if not row["options_enabled"]:
        assert result["initiations"] == 0
        assert result["q_updates"] > 0 and result["beta_updates"] > 0
    if row["options_enabled"]:
        assert result["initiations"] > 0
    result.update(
        {
            "case": row["case"],
            "seed": row["seed"],
            "horizon": row["horizon"],
            "options_enabled": row["options_enabled"],
            "reward_mode": row["reward_mode"],
            "trial": row["trial"],
            "goal_terminates": row["goal_terminates"],
            "maximum_retained_object_bytes": max(allocated),
            "maximum_serialized_state_bytes": max(serialized),
            "final_parameter_touches": state.parameter_touches,
            "final_work_by_class": dict(state.work_by_class),
            "source_identity": identity,
            "resource_measurements": resources,
        }
    )
    package = (config, transitions, audits, accountings, row["steps"])
    return result, package


Mutation = Callable[[list[dict[str, Any]], list[dict[str, Any]]], None]


def mutation_checks(package: Package) -> list[str]:
    config, original_transitions, original_audits, original_accountings, steps = package

    def reward(transitions: list[dict[str, Any]], _: list[dict[str, Any]]) -> None:
        transitions[0]["reward"] += 1

    def work(_: list[dict[str, Any]], values: list[dict[str, Any]]) -> None:
        values[0]["details"]["admission"]["learning"]["accepted"] = False

    def q_delta(_: list[dict[str, Any]], values: list[dict[str, Any]]) -> None:
        audit = next(value for value in values if value["details"]["option_q_deltas"])
        audit["details"]["option_q_deltas"][0]["after"] += 1

    def beta_delta(_: list[dict[str, Any]], values: list[dict[str, Any]]) -> None:
        audit = next(value for value in values if value["details"]["option_beta_deltas"])
        audit["details"]["option_beta_deltas"][0]["after"][1] += 1

    def lineage(_: list[dict[str, Any]], values: list[dict[str, Any]]) -> None:
        audit = next(value for value in values if value["details"]["feature_proposals"])
        audit["details"]["feature_proposals"][0]["artifact_revision_id"] = str(
            __import__("uuid").uuid4()
        )

    def duration(_: list[dict[str, Any]], values: list[dict[str, Any]]) -> None:
        audit = next(
            value
            for value in values
            if any(
                event["kind"] in {"continue", "terminate"}
                for event in value["details"]["option_execution_events"]
            )
        )
        event = next(
            event
            for event in audit["details"]["option_execution_events"]
            if event["kind"] in {"continue", "terminate"}
        )
        event["duration"] += 1

    def track(_: list[dict[str, Any]], values: list[dict[str, Any]]) -> None:
        values[0]["details"]["track"] = "track_b"

    def reward_mode(_: list[dict[str, Any]], values: list[dict[str, Any]]) -> None:
        values[0]["details"]["reward_mode"] = "oblivious"

    def control_metadata(_: list[dict[str, Any]], values: list[dict[str, Any]]) -> None:
        values[0]["details"]["options_enabled"] = False

    mutations: list[tuple[str, Mutation]] = [
        ("reward_tamper", reward),
        ("work_false", work),
        ("q_delta", q_delta),
        ("beta_delta", beta_delta),
        ("feature_lineage", lineage),
        ("option_duration", duration),
        ("track_metadata", track),
        ("reward_mode_metadata", reward_mode),
        ("control_metadata", control_metadata),
    ]
    rejected: list[str] = []
    for name, mutate in mutations:
        transitions = copy.deepcopy(original_transitions)
        audits = copy.deepcopy(original_audits)
        accountings = copy.deepcopy(original_accountings)
        mutate(transitions, audits)
        try:
            reconstruct_records(config, transitions, audits, accountings, steps)
        except (AssertionError, IndexError, KeyError, TypeError, ValueError):
            rejected.append(name)
        else:
            raise AssertionError(f"altered evidence accepted: {name}")
    transitions = copy.deepcopy(original_transitions)
    audits = copy.deepcopy(original_audits)
    accountings = copy.deepcopy(original_accountings)
    transitions.pop()
    audits.pop()
    accountings.pop()
    try:
        reconstruct_records(config, transitions, audits, accountings, steps)
    except (AssertionError, IndexError, KeyError, TypeError, ValueError):
        rejected.append("missing_step")
    else:
        raise AssertionError("altered evidence accepted: missing_step")
    assert len(rejected) == 10
    return rejected


def check_matrix(summary: dict[str, Any], results: list[dict[str, Any]]) -> dict[str, Any]:
    provisional = summary["provisional"]
    assert type(provisional) is bool
    if provisional:
        assert len(results) == 1
    else:
        expected = {
            (seed, enabled, mode, trial, False)
            for seed in (7, 19, 42)
            for enabled, mode, trial in (
                (True, "respecting", 0),
                (True, "respecting", 1),
                (False, "respecting", 0),
                (True, "oblivious", 0),
            )
        }
        expected.add((7, True, "respecting", 0, True))
        actual = {
            (
                result["seed"],
                result["options_enabled"],
                result["reward_mode"],
                result["trial"],
                result["goal_terminates"],
            )
            for result in results
        }
        assert len(results) == len(actual) == len(expected) == 13 and actual == expected
        for seed in (7, 19, 42):
            repeats = [
                result
                for result in results
                if result["seed"] == seed
                and result["options_enabled"]
                and result["reward_mode"] == "respecting"
                and not result["goal_terminates"]
            ]
            assert len(repeats) == 2
            for key in ("trajectory_sha256", "state_sha256", "execution_sha256"):
                assert repeats[0][key] == repeats[1][key]
        durations = {
            duration
            for result in results
            if result["options_enabled"]
            for duration in result["completed_durations"]
        }
        assert len(durations) >= 2
        assert any(result["termination_reasons"].get("learned_beta", 0) for result in results)
        boundary = next(result for result in results if result["goal_terminates"])
        assert boundary["termination_reasons"].get("environment_terminated", 0) > 0
    return {
        "provisional": provisional,
        "cases": len(results),
        "deterministic_repeat_pairs": 0 if provisional else 3,
        "variable_completed_durations": sorted(
            {
                duration
                for result in results
                if result["options_enabled"]
                for duration in result["completed_durations"]
            }
        ),
        "run_limit_censored_invocations": sum(
            result["run_limit_censored_invocations"] for result in results
        ),
    }


def main() -> None:
    assert len(sys.argv) == 2, "usage: python replay.py <root-relative-batch>"
    batch = (ROOT / sys.argv[1]).resolve()
    assert batch.is_relative_to((ROOT / "target").resolve()) and batch.is_dir()
    summary = read_json(batch / "summary.json")
    assert summary["schema"] == "bonsai.online-option-diagnostic/v1"
    assert summary["scientific_quality_claim"] is False
    assert summary["physical_host_acceptance"] is False
    assert summary["resource_profile"]["step_limit"] == 256
    assert summary["resource_profile"]["energy_tier"] == "E0"
    assert summary["resource_profile"]["energy_budget_uj"] is None
    batch_identity = read_json(batch / "source-identity.json")
    assert batch_identity["host"]["physical_acceptance"] is False
    assert batch_identity["host"]["wsl"] is True
    assert lower_hash(summary["operator_sha256"])
    assert summary["operator_sha256"] in batch_identity["executables"].values()
    results: list[dict[str, Any]] = []
    mutation_reference = None
    cases: set[Path] = set()
    for row in summary["runs"]:
        assert set(
            (
                "seed",
                "horizon",
                "steps",
                "options_enabled",
                "reward_mode",
                "trial",
                "goal_terminates",
                "case",
                "runner_exit",
                "receipt_sha256",
                "verifier_exit",
            )
        ) <= set(row)
        assert row["seed"] in {7, 19, 42}
        assert row["horizon"] == 32 and row["steps"] == 256
        assert type(row["options_enabled"]) is bool and type(row["goal_terminates"]) is bool
        assert row["reward_mode"] in {"respecting", "oblivious"}
        case = (ROOT / row["case"]).resolve()
        assert case.parent == batch and case not in cases
        cases.add(case)
        result, package = reconstruct_case(case, row, batch_identity)
        results.append(result)
        if mutation_reference is None and row["options_enabled"] and row["reward_mode"] == "respecting":
            mutation_reference = package
    assert mutation_reference is not None
    rejected = mutation_checks(mutation_reference)
    matrix = check_matrix(summary, results)
    report = {
        "schema": "bonsai.online-option-independent-reconstruction/v1",
        "result": "pass",
        "runs": results,
        "reconstructed_steps": sum(result["steps_reconstructed"] for result in results),
        "tampering_rejected": rejected,
        "matrix": matrix,
        "measurement_scope": {
            "retained_object_bytes": (
                "actual recorded Python-owned learner object graph; checked for positivity, cap, "
                "and admission/audit equality, not independently rederived from canonical state"
            ),
            "agent_rss_bytes": "actual governed cgroup/process observation",
            "serialized_state_bytes": "independently reconstructed canonical JSON bytes",
            "work_items": "declared capacity reservations, not CPU operation measurements",
        },
        "claim_boundary": {
            "scientific_quality_claim": False,
            "physical_host_acceptance": False,
            "track": "factual single-pass Track A",
            "replay_items_retained": 0,
        },
    }
    payload = json.dumps(report, indent=2, sort_keys=True).encode() + b"\n"
    output = batch / "option-replay.json"
    if output.exists():
        assert output.read_bytes() == payload, "non-idempotent option replay output"
    else:
        output.write_bytes(payload)
    print(
        json.dumps(
            {
                "runs": len(results),
                "steps": report["reconstructed_steps"],
                "mutations_rejected": len(rejected),
                "censored": matrix["run_limit_censored_invocations"],
                "result": "pass",
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
