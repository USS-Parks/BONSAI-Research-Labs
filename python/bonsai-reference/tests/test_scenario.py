from __future__ import annotations

from dataclasses import asdict

from bonsai_reference.scenario import (
    ScenarioError,
    ScenarioSpec,
    classify_diagnostic_exposure,
    semantic_stream,
)


def spec() -> ScenarioSpec:
    return ScenarioSpec(
        scenario_id="stable-values-v1",
        version="1.0",
        seed=42,
        horizon=32,
        action_count=3,
        observation_width=4,
        big_world_size=1_000_003,
        change_points=(8, 16, 24),
    )


def test_same_seed_yields_identical_semantic_stream() -> None:
    actions = tuple(step % 3 for step in range(32))
    expected = semantic_stream(spec(), actions)
    for _ in range(100):
        assert semantic_stream(spec(), actions) == expected
    assert expected.public_sha256 == "4be55b7ebb861397b2560c31c3f5c1e62117d7f4eff97913c6c28d1cfb6f16bc"
    assert expected.diagnostic_sha256 == "1f288d732aa14b59f69c44690b6428eff317457da703d92435eab49a5828dd38"


def test_privileged_truth_is_observer_only_and_exposure_forces_track_d() -> None:
    trace = semantic_stream(spec(), (0,) * 32)
    public_fields = set(asdict(trace.public[0]))
    assert public_fields.isdisjoint({"latent_state", "target_action", "world_token"})
    assert trace.diagnostic[0].stream_id == trace.public[0].stream_id
    assert classify_diagnostic_exposure({"privileged_diagnostic_exposed": False}) == "track_a"
    assert classify_diagnostic_exposure({"privileged_diagnostic_exposed": True}) == "track_d"


def test_protocol_rejects_invalid_schedule_and_change_points() -> None:
    try:
        semantic_stream(spec(), (0,))
    except ScenarioError as error:
        assert error.code == "SCENARIO_ACTION_SCHEDULE_INVALID"
    else:
        raise AssertionError("expected invalid schedule")

    base = spec()
    invalid = ScenarioSpec(
        scenario_id=base.scenario_id,
        version=base.version,
        seed=base.seed,
        horizon=base.horizon,
        action_count=base.action_count,
        observation_width=base.observation_width,
        big_world_size=base.big_world_size,
        change_points=(8, 8),
    )
    try:
        semantic_stream(invalid, (0,) * 32)
    except ScenarioError as error:
        assert error.code == "SCENARIO_CHANGE_POINTS_INVALID"
    else:
        raise AssertionError("expected invalid change points")
