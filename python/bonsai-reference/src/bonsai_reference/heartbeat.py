"""Generate the deterministic, platform-qualified M1 heartbeat bundle."""

from __future__ import annotations

import argparse
import hashlib
import json
import mmap
import os
import platform as host_platform
import re
import sys
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import cast

from bonsai_reference.control import run_deterministic_bandit

type JsonValue = None | bool | int | float | str | list[JsonValue] | dict[str, JsonValue]

HORIZON = 32
WORK_BUDGET = 160
EMPTY_SHA256 = hashlib.sha256(b"{}").hexdigest()
METRIC_SPEC_SHA256 = "c61b496da6e23b4722f1b2cb6097faa30857ec95815bac6120a2564f54351b09"
REPOSITORY = "https://github.com/USS-Parks/BONSAI-Research-Labs"
SOURCE_PATTERN = re.compile(r"^[0-9a-f]{40,64}$")


@dataclass(frozen=True, slots=True)
class HeartbeatArtifacts:
    files: dict[str, bytes]
    semantic_summary: dict[str, JsonValue]
    platform_summary: dict[str, JsonValue]


def canonical(value: JsonValue) -> bytes:
    """Return the repository's compact canonical JSON representation."""
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("ascii")


def digest(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def _component(value: JsonValue) -> bytes:
    return canonical(value) + b"\n"


def _normalize_platform(os_family: str, architecture: str) -> tuple[str, str]:
    family = os_family.lower()
    family = {"darwin": "macos", "win32": "windows"}.get(family, family)
    if family not in {"windows", "macos", "linux"}:
        family = "other"
    arch = architecture.lower()
    arch = {"amd64": "x86_64", "x64": "x86_64", "aarch64": "arm64"}.get(arch, arch)
    if not re.fullmatch(r"[a-z0-9][a-z0-9._-]*", arch):
        arch = "unknown"
    return family, arch


def _memory() -> tuple[int, int]:
    # M1 does not claim a physical-memory collector. The schema requires these
    # positive shape fields, so one page is recorded and explicitly unattested.
    return mmap.PAGESIZE, mmap.PAGESIZE


def _platform_inventory(os_family: str, architecture: str, cargo_lock_hash: str) -> dict[str, JsonValue]:
    physical_memory, page_size = _memory()
    identity = str(uuid.uuid5(uuid.NAMESPACE_URL, f"bonsai-m1:{os_family}:{architecture}"))
    processor = host_platform.processor() or "unavailable"
    return {
        "schema_version": "1.0",
        "inventory_id": identity,
        "machine_identity_id": identity,
        "os": {
            "family": os_family,
            "version": host_platform.release() or "unavailable",
            "build": host_platform.version() or "unavailable",
            "kernel": host_platform.system() or "unavailable",
            "architecture": architecture,
        },
        "cpu": {
            "vendor": "hosted-runner-unattested",
            "model": processor,
            "architecture": architecture,
            "logical_cores": os.cpu_count() or 1,
        },
        "accelerators": [],
        "memory": {"physical_total_bytes": physical_memory, "page_size_bytes": page_size},
        "clocks": [{"clock_id": "python_monotonic", "kind": "monotonic", "resolution_ns": 1, "monotonic": True}],
        "drivers": [],
        "runtimes": [{"component_id": "python", "version": host_platform.python_version()}],
        "compilers": [{"component_id": "rustc", "version": "1.96.0"}],
        "dependency_locks": [{"ecosystem": "cargo", "lockfile_name": "Cargo.lock", "sha256": cargo_lock_hash}],
        "privilege": {"process_level": "unknown", "elevation_available": False},
        "collectors": [
            {
                "collector_id": "m1-semantic-accounting",
                "version": "1.0",
                "status": "available",
                "privilege_requirement": "none",
                "capabilities": ["environment_steps", "operation_counts"],
            }
        ],
        "thermal_power": {"thermal_state": "unavailable", "power_source": "unknown"},
    }


def _experiment_manifest(source_revision: str) -> dict[str, JsonValue]:
    return {
        "schema_version": "1.0",
        "manifest_id": "11111111-1111-4111-8111-111111111113",
        "run_id": "22222222-2222-4222-8222-222222222223",
        "source": {"repository": REPOSITORY, "revision": source_revision, "dirty": False},
        "adapter": {
            "component_id": "bonsai-reference-primitive-control",
            "version": "1.0.0",
            "entrypoint": ["python", "-m", "bonsai_reference.heartbeat"],
            "config": {"batch_size": 1, "replay_capacity": 0},
        },
        "environment": {
            "component_id": "bonsai-deterministic-bandit",
            "version": "1.0.0",
            "entrypoint": ["bonsai_reference.control", "run_deterministic_bandit"],
            "config": {"target_action": 2},
        },
        "seeds": [{"seed_id": "environment", "value": "0"}],
        "track": {
            "declared_track": "A",
            "batch_size": 1,
            "replay": {"enabled": False, "source": "none", "capacity_transitions": 0},
            "offline_updates": False,
            "observer_data_access": False,
            "privileged_inputs": False,
            "human_labels": False,
        },
        "resource_profile": {
            "profile_id": "S",
            "step_limit": HORIZON,
            "wall_time_limit_ns": 120_000_000_000,
            "agent_rss_limit_bytes": 1_073_741_824,
            "agent_storage_limit_bytes": 67_108_864,
            "observer_output_limit_bytes": 536_870_912,
            "per_step_cpu_time_limit_ns": 10_000_000,
            "action_deadline_ns": 50_000_000,
            "energy_tier": "E0",
            "energy_budget_uj": None,
        },
        "metrics": [{"metric_id": "behavior.mean_reward", "version": "1.0", "required": True, "parameters": {}}],
        "scenario": {
            "scenario_id": "diagnostic.stable-bandit",
            "version": "1.0.0",
            "family": "stationary_control",
            "variant": "m1-heartbeat",
            "reward_unit_id": "scenario_reward",
            "config": {"horizon": HORIZON, "target_action": 2},
        },
        "expected_counters": [
            {"counter_id": "environment_steps", "unit": "1", "acceptable_basis": "measured", "required_for_run": True},
            {"counter_id": "work_items", "unit": "1", "acceptable_basis": "measured", "required_for_run": True},
        ],
        "publication_eligibility": {"status": "eligible", "reason_codes": []},
    }


def _track_declaration() -> dict[str, JsonValue]:
    return {
        "schema_version": "1.0",
        "declared_track": "A",
        "runtime_facts_complete": True,
        "batch_size": 1,
        "transition_access": "single_pass",
        "replay_capacity_transitions": 0,
        "offline_updates": False,
        "observer_data_access": False,
        "privileged_state": False,
        "human_labels": False,
        "domain_feature_targets": False,
        "update_schedule": "event_driven",
        "fixed_external_budgets": True,
    }


def _load_resource_policy(repository_root: Path) -> JsonValue:
    raw = json.loads((repository_root / "fixtures/resource-policy/v1/valid.json").read_text(encoding="utf-8"))
    return cast(JsonValue, raw)


def build_heartbeat(
    repository_root: Path,
    source_revision: str,
    os_family: str,
    architecture: str,
) -> HeartbeatArtifacts:
    """Build one deterministic heartbeat without writing to disk."""
    if SOURCE_PATTERN.fullmatch(source_revision) is None:
        raise ValueError("HEARTBEAT_SOURCE_REVISION_INVALID")
    family, arch = _normalize_platform(os_family, architecture)
    trace = run_deterministic_bandit(HORIZON)
    work_used = trace.accounting.work_items
    decision = "admit" if work_used <= WORK_BUDGET else "reject"
    headroom = max(WORK_BUDGET - work_used, 0)

    actions: JsonValue = list(trace.actions)
    rewards: JsonValue = list(trace.rewards)
    cumulative: JsonValue = list(trace.cumulative_reward)
    event_segment: JsonValue = {
        "schema": "bonsai.m1-heartbeat-events/v1",
        "actions": actions,
        "rewards": rewards,
        "cumulative_reward": cumulative,
    }
    event_bytes = _component(event_segment)
    event_hash = digest(event_bytes)
    cargo_lock_hash = digest((repository_root / "Cargo.lock").read_bytes())
    platform_inventory = _platform_inventory(family, arch, cargo_lock_hash)

    components: list[tuple[str, str, JsonValue]] = [
        ("experiment.json", "experiment_manifest", _experiment_manifest(source_revision)),
        ("track.json", "track_declaration", _track_declaration()),
        ("inventory.json", "platform_inventory", platform_inventory),
        ("resource-policy.json", "resource_policy", _load_resource_policy(repository_root)),
        ("failures.json", "failure_log", []),
        (
            "metric-estimate.json",
            "metric_estimate",
            {
                "schema_version": "1.0",
                "estimate_id": "10000000-0000-4000-8000-000000000003",
                "run_id": "22222222-2222-4222-8222-222222222223",
                "metric_spec": {
                    "metric_id": "behavior.mean_reward",
                    "metric_version": "1.0",
                    "canonical_sha256": METRIC_SPEC_SHA256,
                },
                "population": {
                    "population_id": "authorized_transitions",
                    "eligible_count": HORIZON,
                    "observed_count": HORIZON,
                    "selection_sha256": digest(canonical(actions)),
                },
                "window": {"basis": "step_count", "start": 0, "end": HORIZON},
                "result": {
                    "kind": "scalar",
                    "value": trace.cumulative_reward[-1] / HORIZON,
                    "unit": "scenario_reward/step",
                    "availability": "measured",
                },
                "estimator": {
                    "estimator_id": "arithmetic_mean",
                    "estimator_version": "1.0",
                    "parameters_sha256": EMPTY_SHA256,
                },
                "missingness": {
                    "missing_count": 0,
                    "invalid_count": 0,
                    "coverage_ratio": 1.0,
                    "disposition": "complete",
                    "reason_codes": [],
                },
                "precision": {
                    "representation": "finite_binary64",
                    "significant_bits": 53,
                    "absolute_tolerance": 0.0,
                    "relative_tolerance": 0.0,
                },
                "uncertainty_ids": ["30000000-0000-4000-8000-000000000003"],
                "inputs": [
                    {
                        "evidence_id": "m1-heartbeat-events",
                        "evidence_type": "event",
                        "sha256": event_hash,
                        "role": "primitive actions and rewards",
                    }
                ],
            },
        ),
        ("events.json", "event_segment", event_segment),
    ]
    files = {name: _component(value) for name, _role, value in components}
    manifest_entries: list[JsonValue] = [
        {"path": name, "sha256": digest(files[name]), "role": role, "required": True}
        for name, role, _value in components
    ]
    manifest: JsonValue = {
        "format": "bonsai.bundle/v1",
        "epoch": 1,
        "minor": 0,
        "bundle_id": "90000000-0000-4000-8000-000000000003",
        "files": manifest_entries,
        "migration": {"status": "current", "source_epoch": 1, "registry_id": "bonsai.bundle-migrations/v1"},
    }
    files["manifest.json"] = _component(manifest)

    semantic_summary: dict[str, JsonValue] = {
        "schema": "bonsai.m1-heartbeat-summary/v1",
        "scenario": "diagnostic.stable-bandit",
        "horizon": HORIZON,
        "actions_sha256": digest(canonical(actions)),
        "rewards_sha256": digest(canonical(rewards)),
        "cumulative_reward": trace.cumulative_reward[-1],
        "track": "A",
        "batch_size": 1,
        "replay_capacity": 0,
        "environment_steps": trace.accounting.environment_steps,
        "updates": trace.accounting.updates,
        "parameter_touches": trace.accounting.parameter_touches,
        "work_items": work_used,
        "work_budget": WORK_BUDGET,
        "budget_decision": decision,
        "budget_headroom": headroom,
        "overhead": {
            "basis": "deterministic_semantic_fixture",
            "throughput_point_overhead_ppm": 0,
            "p95_latency_point_overhead_ppm": 0,
            "physical_acceptance": False,
        },
        "claims": {
            "C0": "reportable_evidence_present",
            "C1": "reportable_budget_evidence_present",
            "verdict": "not_adjudicated",
            "rule_version": "1.0.0",
        },
    }
    platform_summary: dict[str, JsonValue] = {
        "schema": "bonsai.m1-platform-row/v1",
        "os_family": family,
        "architecture": arch,
        "inventory_id": platform_inventory["inventory_id"],
        "thermal_state": "unavailable",
        "energy_tier": "E0",
    }
    hashes: dict[str, JsonValue] = {name: digest(content) for name, content in sorted(files.items())}
    report_input: dict[str, JsonValue] = {
        "schema": "bonsai.static-report/v1",
        "title": "BONSAI M1 auditable heartbeat",
        "manifest": {"bundle_id": "90000000-0000-4000-8000-000000000003", "source_revision": source_revision},
        "platform": platform_summary,
        "track": {"declared": "A", "derived": "A", "reason": "STRICT_EXPERIENTIAL_FACTS"},
        "resources": {
            "work_budget": WORK_BUDGET,
            "work_items": work_used,
            "headroom": headroom,
            "decision": decision,
            "energy_tier": "E0",
        },
        "overhead": semantic_summary["overhead"],
        "behavior": {
            "actions": actions,
            "rewards": rewards,
            "cumulative_reward": trace.cumulative_reward[-1],
            "mean_reward": trace.cumulative_reward[-1] / HORIZON,
        },
        "failures": [],
        "claims": semantic_summary["claims"],
        "limitations": [
            "Hosted semantic fixture; no physical-host, energy, thermal, or hard process-enforcement acceptance.",
            "C0/C1 evidence is reportable but claim adjudication remains BV-04; no C0-C5 pass is asserted.",
        ],
        "hashes": hashes,
    }
    files["report-input.json"] = _component(report_input)
    files["semantic-summary.json"] = _component(semantic_summary)
    files["platform-summary.json"] = _component(platform_summary)
    files["lineage.json"] = _component({"source_revision": source_revision, "parents": []})
    files["metrics.json"] = _component({"mean_reward": trace.cumulative_reward[-1] / HORIZON})
    files["decisions.json"] = _component({"budget": decision, "work_items": work_used, "limit": WORK_BUDGET})
    files["comparisons.json"] = _component({"comparator": "none", "reason": "M1_PRIMITIVE_HEARTBEAT"})
    return HeartbeatArtifacts(files=files, semantic_summary=semantic_summary, platform_summary=platform_summary)


def emit(artifacts: HeartbeatArtifacts, output: Path) -> None:
    """Write one freshly generated bundle directory."""
    output.mkdir(parents=True, exist_ok=True)
    for name, content in artifacts.files.items():
        (output / name).write_bytes(content)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--source-revision", required=True)
    parser.add_argument("--os-family", default=sys.platform)
    parser.add_argument("--architecture", default=host_platform.machine())
    parser.add_argument("--repository-root", type=Path, default=Path.cwd())
    return parser


def main() -> None:
    arguments = _parser().parse_args()
    artifacts = build_heartbeat(
        arguments.repository_root,
        arguments.source_revision,
        arguments.os_family,
        arguments.architecture,
    )
    emit(artifacts, arguments.output)
    print(f"m1 heartbeat semantic_sha256={digest(canonical(artifacts.semantic_summary))}")


if __name__ == "__main__":
    main()
