"""Small continuing-chain diagnostic through the existing environment protocol."""
from __future__ import annotations

import argparse
import hashlib
import json
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import cast

from bonsai_reference.adapter_protocol import serve
from bonsai_reference.environment_adapter import MAXIMUM, SessionAdapter
from bonsai_reference.scenario import ScenarioError, SessionObservation, SessionTransition


@dataclass(frozen=True, slots=True)
class ChainSpec:
    scenario_id: str
    version: str
    seed: int
    horizon: int
    action_count: int
    observation_width: int
    size: int
    reward_state: int
    goal_terminates: bool

    def validate(self) -> None:
        if (self.scenario_id != "feature-attainment-chain" or self.version != "1.0"
                or type(self.seed) is not int or not 0 <= self.seed < 2**64
                or type(self.horizon) is not int or not 1 <= self.horizon <= 2000
                or type(self.action_count) is not int or self.action_count != 2
                or type(self.observation_width) is not int or self.observation_width != 1
                or type(self.size) is not int or self.size != 5
                or type(self.reward_state) is not int or self.reward_state != 4
                or type(self.goal_terminates) is not bool):
            raise ScenarioError("CHAIN_CONFIGURATION_INVALID")


class ChainSession:
    """Reward parameters remain environment-side; only the current position is public."""

    def __init__(self, spec: ChainSpec) -> None:
        spec.validate()
        self._spec = spec
        self._position = 0
        self._step = 0
        self._done = False
        self._stream = ""

    def reset(self, seed: int) -> SessionObservation:
        if type(seed) is not int or not 0 <= seed < 2**64:
            raise ScenarioError("SCENARIO_SEED_INVALID")
        self._position = seed % (self._spec.size - 1)
        self._step, self._done = 0, False
        self._stream = hashlib.sha256(json.dumps(
            {**asdict(self._spec), "seed": seed}, sort_keys=True, separators=(",", ":"),
        ).encode()).hexdigest()
        return self.observe()

    def observe(self) -> SessionObservation:
        if not self._stream:
            raise ScenarioError("SCENARIO_RESET_REQUIRED")
        return SessionObservation(self._stream, self._step, (self._position,), () if self._done else (0, 1))

    def step(self, index: int, action: int) -> SessionTransition:
        if not self._stream:
            raise ScenarioError("SCENARIO_RESET_REQUIRED")
        if self._done:
            raise ScenarioError("SCENARIO_EPISODE_FINISHED")
        if type(index) is not int or index != self._step:
            raise ScenarioError("SCENARIO_STEP_OUT_OF_ORDER")
        if type(action) is not int or action not in (0, 1):
            raise ScenarioError("SCENARIO_ACTION_INVALID")
        self._position = (self._position + (1 if action == 0 else -1)) % self._spec.size
        self._step += 1
        attained = self._position == self._spec.reward_state
        terminated = attained and self._spec.goal_terminates
        truncated = not terminated and self._step == self._spec.horizon
        self._done = terminated or truncated
        return SessionTransition(index, action, 4 if attained else -1, self.observe(), terminated, truncated)


def load_spec(path: Path) -> tuple[ChainSpec, bytes]:
    with path.open("rb") as source:
        payload = source.read(MAXIMUM + 1)
    if len(payload) > MAXIMUM:
        raise ScenarioError("SCENARIO_CONFIGURATION_TOO_LARGE")
    raw: object = json.loads(payload)
    if not isinstance(raw, dict):
        raise ScenarioError("CHAIN_CONFIGURATION_INVALID")
    value = cast(dict[str, object], raw)
    if set(value) != set(ChainSpec.__dataclass_fields__):
        raise ScenarioError("CHAIN_CONFIGURATION_INVALID")
    spec = ChainSpec(
        scenario_id=cast(str, value["scenario_id"]), version=cast(str, value["version"]),
        seed=cast(int, value["seed"]), horizon=cast(int, value["horizon"]),
        action_count=cast(int, value["action_count"]), observation_width=cast(int, value["observation_width"]),
        size=cast(int, value["size"]), reward_state=cast(int, value["reward_state"]),
        goal_terminates=cast(bool, value["goal_terminates"]),
    )
    spec.validate()
    return spec, hashlib.sha256(payload).digest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", type=Path, required=True)
    arguments = parser.parse_args()
    spec, digest = load_spec(arguments.spec)
    return serve(SessionAdapter(ChainSession(spec), digest))


if __name__ == "__main__":
    raise SystemExit(main())
