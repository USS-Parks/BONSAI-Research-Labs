from __future__ import annotations

import json
from dataclasses import asdict, replace
from pathlib import Path

import pytest
from bonsai_reference.scenario import ScenarioError, ScenarioSession, ScenarioSpec


def spec() -> ScenarioSpec:
    return ScenarioSpec("causal-ring-diagnostic", "1.0", 42, 8, 3, 4, 17, ())


def target(session: ScenarioSession) -> int:
    truth = session.diagnostic()
    return (truth.goal_state - truth.latent_state) % 17 - 1


def test_actions_change_the_next_state_and_prescribed_outcome() -> None:
    left, right = ScenarioSession(spec()), ScenarioSession(spec())
    assert left.reset(42) == right.reset(42)
    initial = left.diagnostic()
    winning = target(left)
    losing = (winning + 1) % 3
    success, alternative = left.step(0, winning), right.step(0, losing)
    assert success.reward == 1 and success.terminated and not success.truncated
    assert alternative.reward == 0 and not alternative.terminated and not alternative.truncated
    assert left.diagnostic().latent_state == (initial.latent_state + winning + 1) % 17
    assert right.diagnostic().latent_state == (initial.latent_state + losing + 1) % 17
    assert success.next.observation != alternative.next.observation
    assert success.next.allowed_actions == ()
    assert alternative.next.allowed_actions == (0, 1, 2)


def test_horizon_is_truncation_and_terminal_wins_at_the_boundary() -> None:
    left, right = ScenarioSession(replace(spec(), horizon=1)), ScenarioSession(replace(spec(), horizon=1))
    left.reset(42)
    right.reset(42)
    winning = target(left)
    terminal = left.step(0, winning)
    truncated = right.step(0, (winning + 1) % 3)
    assert terminal.terminated and not terminal.truncated
    assert truncated.truncated and not truncated.terminated
    with pytest.raises(ScenarioError, match="SCENARIO_EPISODE_FINISHED"):
        right.step(1, 0)


def test_invalid_steps_are_atomic_and_reset_is_reproducible() -> None:
    session, reference = ScenarioSession(spec()), ScenarioSession(spec())
    with pytest.raises(ScenarioError, match="SCENARIO_RESET_REQUIRED"):
        session.step(0, 0)
    original = session.reset(42)
    assert reference.reset(42) == original
    for index, action, code in [(1, 0, "STEP_OUT_OF_ORDER"), (0, -1, "ACTION_INVALID"), (0, 3, "ACTION_INVALID")]:
        with pytest.raises(ScenarioError, match=code):
            session.step(index, action)
        assert session.observe() == original
        assert session.diagnostic() == reference.diagnostic()
    winning = target(session)
    assert session.step(0, winning) == reference.step(0, winning)
    with pytest.raises(ScenarioError, match="SCENARIO_EPISODE_FINISHED"):
        session.step(1, winning)
    assert session.reset(42) == original
    reference.reset(42)
    assert session.step(0, winning) == reference.step(0, winning)


def test_observation_queries_do_not_advance_randomness_and_truth_is_separate() -> None:
    left, right = ScenarioSession(spec()), ScenarioSession(spec())
    left.reset(7)
    right.reset(7)
    for _ in range(10):
        assert left.observe() == right.observe()
    first, second = left.step(0, 0), right.step(0, 0)
    assert first == second
    assert set(asdict(first.next)) == {"stream_id", "step", "observation", "allowed_actions"}
    assert set(asdict(left.diagnostic())) == {"stream_id", "step", "latent_state", "goal_state"}


def test_seed_and_shape_limits_reject_without_allocating_large_spaces() -> None:
    session = ScenarioSession(spec())
    for seed in [-1, 1 << 64, True]:
        with pytest.raises(ScenarioError, match="SCENARIO_SEED_INVALID"):
            session.reset(seed)
    with pytest.raises(ScenarioError, match="SCENARIO_SESSION_LIMIT_INVALID"):
        ScenarioSession(replace(spec(), observation_width=1_000_000_000))


def test_public_and_observer_golden_traces_replay_without_future_schedule_input() -> None:
    folder = Path(__file__).resolve().parents[3] / "fixtures/scenario/causal-v1"
    expected_public = json.loads((folder / "public-trace.json").read_text(encoding="utf-8"))
    expected_diagnostic = json.loads((folder / "diagnostic-trace.json").read_text(encoding="utf-8"))
    for extra_queries in range(2):
        session = ScenarioSession(spec())
        initial = asdict(session.reset(42))
        diagnostic = [asdict(session.diagnostic())]
        transitions: list[dict[str, object]] = []
        for index in range(spec().horizon):
            for _ in range(extra_queries):
                session.observe()
            transition = session.step(index, index % spec().action_count)
            transitions.append(asdict(transition))
            diagnostic.append(asdict(session.diagnostic()))
            if transition.terminated or transition.truncated:
                break
        public = {"initial": initial, "transitions": transitions}
        assert json.loads(json.dumps(public)) == expected_public
        assert json.loads(json.dumps(diagnostic)) == expected_diagnostic
