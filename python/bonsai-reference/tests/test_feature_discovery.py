from __future__ import annotations

import os
import sys
import uuid
from pathlib import Path
from typing import cast

import pytest
from bonsai_reference.brdc1 import CycleError
from bonsai_reference.feature_admission import FeatureGovernor
from bonsai_reference.feature_discovery import OnlineFeatureStage, canonical, retained_bytes

ROOT = Path(__file__).resolve().parents[3]
DEFAULT = ROOT / ("target/debug/examples/feature_admission.exe" if sys.platform == "win32"
                  else "target/debug/examples/feature_admission")
EXECUTABLE = Path(os.environ.get("BONSAI_FEATURE_GOVERNOR", str(DEFAULT)))
pytestmark = pytest.mark.skipif(not EXECUTABLE.is_file(), reason="build the feature_admission Rust example")


def event_id(step: int) -> str:
    return str(uuid.uuid5(uuid.NAMESPACE_URL, f"feature-test-event:{step}"))


def run(seed: int, enabled: bool = True) -> tuple[OnlineFeatureStage, list[dict[str, object]]]:
    stage = OnlineFeatureStage(2, seed, enabled=enabled)
    governor = FeatureGovernor(EXECUTABLE)
    results: list[dict[str, object]] = []
    try:
        for step in range(24):
            result = stage.observe(step, (step % 2, (step + 1) % 2), int(step % 3 != 0),
                                   event_id(step), governor)
            assert result["accepted"]
            assert result["serialized_bytes"] == len(canonical(stage.online_snapshot()))
            assert result["allocated_bytes"] == sys.getsizeof(stage) + retained_bytes(vars(stage))
            results.append(result)
    finally:
        governor.close()
    return stage, results


def test_proposals_and_revisions_are_reproducible_from_experience() -> None:
    first, trace = run(7)
    repeated, _ = run(7)
    assert canonical(first.online_snapshot()) == canonical(repeated.online_snapshot())
    events = [event for row in trace for event in cast(list[dict[str, object]], row["events"])]
    assert {event["kind"] for event in events} == {"birth", "revision"}
    assert all(event["utility"] is None for event in events)
    latest: dict[object, object] = {}
    for event in events:
        if event["kind"] == "revision":
            assert event["previous_revision_id"] == latest[event["artifact_id"]]
        latest[event["artifact_id"]] = event["artifact_revision_id"]
        exposure = cast(dict[str, object], event["exposure"])
        assert cast(int, exposure["count"]) >= 2
        assert cast(int, exposure["reward_sum"]) > 0
    assert any(value > 0 for _, value in first.evaluate((0, 1)))


def test_disabled_control_changes_only_feature_construction() -> None:
    enabled, rows = run(42)
    disabled, control = run(42, False)
    assert enabled.snapshot()
    assert disabled.snapshot() == ()
    assert disabled.online_snapshot()["statistics"] == []
    assert len(rows) == len(control) == 24
    assert disabled.online_snapshot()["next_step"] == enabled.online_snapshot()["next_step"]
    assert all(row["events"] == [] for row in control)


@pytest.mark.parametrize("limits,reason", [
    ({"work_limit": 1}, "WORK_CLASS_RESERVATION_EXHAUSTED"),
    ({"memory_limit": 1}, "FEATURE_RETAINED_MEMORY_LIMIT"),
    ({"serialized_limit": 1}, "STORAGE_BYTE_BUDGET_EXHAUSTED"),
])
def test_live_denials_cannot_mutate_feature_or_exposure_state(limits: dict[str, int], reason: str) -> None:
    stage = OnlineFeatureStage(2, 7)
    before = canonical(stage.online_snapshot())
    governor = FeatureGovernor(EXECUTABLE, **limits)
    try:
        result = stage.observe(0, (1, 2), 1, event_id(0), governor)
    finally:
        governor.close()
    assert not result["accepted"] and result["reason_code"] == reason
    assert result["state_before"] == result["state_after"]
    assert canonical(stage.online_snapshot()) == before
    assert result["events"] == []
    accepted = FeatureGovernor(EXECUTABLE)
    try:
        assert stage.observe(0, (1, 2), 1, event_id(0), accepted)["accepted"]
    finally:
        accepted.close()


def test_retained_statistics_and_features_stay_bounded() -> None:
    stage = OnlineFeatureStage(1, 19, max_statistics=4, max_features=2)
    governor = FeatureGovernor(EXECUTABLE)
    try:
        for step in range(80):
            value = step // 8
            result = stage.observe(step, (value,), 1, event_id(step), governor)
            assert result["accepted"]
            assert cast(int, result["statistics_count"]) <= 4
            assert cast(int, result["feature_count"]) <= 2
        for record in stage.snapshot():
            assert record.bytes == len(canonical(record.representation))
            assert record.bytes != len(record.representation)
        assert stage.online_snapshot()["replay_items_retained"] == 0
    finally:
        governor.close()


def test_nonpositive_experience_does_not_receive_supplied_utility() -> None:
    stage = OnlineFeatureStage(1, 19)
    governor = FeatureGovernor(EXECUTABLE)
    try:
        for step in range(8):
            assert stage.observe(step, (2,), 0, event_id(step), governor)["accepted"]
        assert stage.snapshot() == ()
    finally:
        governor.close()

@pytest.mark.parametrize("operation", ["produce", "revise", "retire", "consumer", "utility"])
def test_inherited_mutation_cannot_bypass_online_admission(operation: str) -> None:
    stage, _ = run(7)
    feature = stage.snapshot()[0].feature_id
    before = canonical(stage.online_snapshot())
    with pytest.raises(CycleError, match="FEATURE_ONLINE_OBSERVATION_REQUIRED"):
        if operation == "produce":
            stage.produce("injected", (9, 9), 24)
        elif operation == "revise":
            stage.revise(feature, (9, 9), 24)
        elif operation == "retire":
            stage.retire(feature, 24)
        elif operation == "consumer":
            stage.add_consumer(feature)
        else:
            stage.set_utility(feature, 100)
    assert canonical(stage.online_snapshot()) == before


@pytest.mark.parametrize("setting", ["width", "seed", "enabled", "max_statistics", "max_features"])
def test_snapshot_and_configuration_cannot_mutate_live_state(setting: str) -> None:
    stage, _ = run(7)
    before = canonical(stage.online_snapshot())
    snapshot = stage.online_snapshot()
    versions = cast(dict[str, object], snapshot["versions"])
    versions.clear()
    with pytest.raises(AttributeError):
        setattr(stage, setting, 1_000_000)
    assert canonical(stage.online_snapshot()) == before


def test_early_governor_exit_is_reaped_without_masking_transport_error() -> None:
    governor = FeatureGovernor(EXECUTABLE, work_limit=0)
    try:
        with pytest.raises((OSError, ValueError)):
            governor.work("feature-work-0", 10)
    finally:
        governor.close()
    assert governor.returncode is not None and governor.returncode != 0
    governor.close()

@pytest.mark.parametrize("limits", [
    {"work_limit": 1}, {"memory_limit": 1}, {"serialized_limit": 1},
])
def test_live_denial_at_revision_keeps_populated_state(limits: dict[str, int]) -> None:
    import copy

    stage, _ = run(7)
    preparing = FeatureGovernor(EXECUTABLE)
    try:
        for step in range(24, 27):
            assert stage.observe(step, (0, 1), 1, event_id(step), preparing)["accepted"]
    finally:
        preparing.close()
    expected = copy.deepcopy(stage)
    proving = FeatureGovernor(EXECUTABLE)
    try:
        would_update = expected.observe(27, (0, 1), 1, event_id(27), proving)
        assert any(event["kind"] == "revision"
                   for event in cast(list[dict[str, object]], would_update["events"]))
    finally:
        proving.close()
    before = canonical(stage.online_snapshot())
    refusing = FeatureGovernor(EXECUTABLE, **limits)
    try:
        denied = stage.observe(27, (0, 1), 1, event_id(27), refusing)
    finally:
        refusing.close()
    assert not denied["accepted"] and denied["events"] == []
    assert canonical(stage.online_snapshot()) == before
