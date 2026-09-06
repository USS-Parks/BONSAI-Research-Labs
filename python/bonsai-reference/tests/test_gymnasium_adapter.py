from __future__ import annotations

import importlib
import importlib.util
import json
from dataclasses import asdict
from pathlib import Path
from typing import cast

import pytest
from bonsai.adapter.v1 import adapter_pb2 as wire
from bonsai_reference.adapter_wire import decode_into, encode
from bonsai_reference.gymnasium_session import GymnasiumAPI, GymnasiumSession, GymnasiumSpec
from bonsai_reference.scenario import ScenarioError
from test_environment_adapter import ACT, ROOT, LiveEnvironment

AVAILABLE = importlib.util.find_spec("gymnasium") is not None


def test_optional_dependency_is_loaded_only_when_constructing_external_session() -> None:
    spec = GymnasiumSpec()
    spec.validate()
    if not AVAILABLE:
        with pytest.raises(ScenarioError, match="GYMNASIUM_OPTIONAL_DEPENDENCY_MISSING"):
            GymnasiumSession(spec)


@pytest.mark.skipif(not AVAILABLE, reason="optional Gymnasium extra is absent from the core installation")
@pytest.mark.parametrize("seed", [0, 42, 9223372036854775808])
@pytest.mark.parametrize("horizon,actions,ending", [
    (8, [2, 2, 1, 1, 1, 2], (True, False)),
    (3, [0, 0, 0], (False, True)),
    (2, [1, 2], (True, True)),
])
def test_direct_and_live_adapter_match(
    tmp_path: Path, seed: int, horizon: int, actions: list[int], ending: tuple[bool, bool],
) -> None:
    spec = GymnasiumSpec(horizon=horizon)
    configuration = tmp_path / "spec.json"
    configuration.write_text(json.dumps(asdict(spec)))
    gym = cast(GymnasiumAPI, cast(object, importlib.import_module("gymnasium")))
    direct = gym.make("FrozenLake-v1", map_name="4x4", is_slippery=False,
                      max_episode_steps=horizon, render_mode=None)
    live = LiveEnvironment(configuration, [
        "-I", "-B", str(ROOT / "scripts/adapter_entrypoint.py"), "gymnasium_adapter",
        "--spec", str(configuration),
    ])
    trace: list[dict[str, object]] = []
    try:
        live.configure()
        response = live.exchange(wire.AdapterFrame(reset=wire.Reset(
            episode_id=bytes([2] * 16), deterministic_seed=seed, deadline_monotonic_ns=live.deadline(),
        )))
        assert response.ack.operation == wire.OPERATION_RESET
        initial, _info = direct.reset(seed=seed)
        observation = live.observe()
        assert list(observation.observation) == [initial]
        assert list(observation.allowed_actions) == [0, 1, 2, 3]
        transition = wire.CausalTransition()
        for index, action in enumerate(actions):
            expected, reward, terminated, truncated, _info = direct.step(action)
            response = live.step(ACT, encode(wire.CausalAction(step=index, action=action)))
            transition = wire.CausalTransition()
            decode_into(transition, response.step_result.action)
            assert transition.step == index and transition.action == action
            assert transition.reward == reward
            assert transition.terminated == terminated and transition.truncated == truncated
            assert list(transition.next.observation) == [expected]
            assert list(transition.next.allowed_actions) == ([] if terminated or truncated else [0, 1, 2, 3])
            trace.append({"step": index, "action": action, "observation": list(transition.next.observation),
                          "reward": transition.reward, "terminated": transition.terminated,
                          "truncated": transition.truncated, "transition_hex": encode(transition).hex()})
        assert (transition.terminated, transition.truncated) == ending
        rejected = live.step(ACT, encode(wire.CausalAction(step=len(actions), action=0)))
        assert rejected.error.reason_code == "SCENARIO_EPISODE_FINISHED"
        assert live.child.wait(timeout=2) == 2
        (tmp_path / "direct-versus-adapter.json").write_text(json.dumps({
            "seed": seed, "spec": asdict(spec), "trace": trace, "protocol": live.transcript,
        }, indent=2))
    finally:
        live.close()
