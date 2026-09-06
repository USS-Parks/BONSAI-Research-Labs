"""Online causal environment on the existing framed Protobuf process protocol."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import cast

from bonsai.adapter.v1 import adapter_pb2 as wire

from bonsai_reference.adapter_protocol import OrderedAdapter, serve
from bonsai_reference.adapter_wire import decode_into, encode
from bonsai_reference.scenario import ScenarioError, ScenarioSession, ScenarioSpec, SessionObservation

MAXIMUM = 65_536
OBSERVE = "bonsai.environment.observe/v1"
ACT = "bonsai.environment.action/v1"


def capabilities() -> wire.CapabilityDeclaration:
    return wire.CapabilityDeclaration(
        reset=True, work=False, feedback=False, asynchronous_events=False,
        accepted_input_types=[OBSERVE, ACT], emitted_event_types=[],
        retains_transitions=False, offline_updates=False, observer_data_access=False,
        privileged_state_access=False, filesystem_read=True, filesystem_write=False, network_access=False,
    )


def public_observation(value: SessionObservation) -> wire.CausalObservation:
    """An explicit projection; diagnostic fields cannot enter this message."""
    return wire.CausalObservation(
        stream_id=value.stream_id, step=value.step,
        observation=value.observation, allowed_actions=value.allowed_actions,
    )


class EnvironmentAdapter(OrderedAdapter):
    def __init__(self, spec: ScenarioSpec, configuration_sha256: bytes) -> None:
        super().__init__(capabilities(), configuration_sha256)
        self.session = ScenarioSession(spec)
        self._query = 0
        self._observed = False

    def _reset(self, seed: int) -> None:
        self.session.reset(seed)
        self._query, self._observed = 0, False

    def _step(self, request: wire.Step) -> wire.AdapterFrame:
        if request.step_index != self._query:
            raise ScenarioError("ADAPTER_STEP_OUT_OF_ORDER")
        if hashlib.sha256(request.input).digest() != request.input_sha256:
            raise ScenarioError("ADAPTER_INPUT_DIGEST_INVALID")
        self._check_deadline(request.deadline_monotonic_ns)
        if request.input_type == OBSERVE and not self._observed and request.input == b"":
            result = encode(public_observation(self.session.observe()))
            self._observed = True
        elif request.input_type == ACT and self._observed:
            action = wire.CausalAction()
            decode_into(action, request.input)
            transition = self.session.step(action.step, action.action)
            result = encode(wire.CausalTransition(
                step=transition.step, action=transition.action, reward=transition.reward,
                next=public_observation(transition.next),
                terminated=transition.terminated, truncated=transition.truncated,
            ))
        else:
            raise ScenarioError("ADAPTER_INPUT_TYPE_OR_ORDER_INVALID")
        self._query += 1
        return wire.AdapterFrame(step_result=wire.StepResult(
            step_index=request.step_index, action=result, action_sha256=hashlib.sha256(result).digest(),
        ))


def load_spec(path: Path) -> tuple[ScenarioSpec, bytes]:
    with path.open("rb") as source:
        payload = source.read(MAXIMUM + 1)
    if len(payload) > MAXIMUM:
        raise ScenarioError("SCENARIO_CONFIGURATION_TOO_LARGE")
    raw: object = json.loads(payload)
    if not isinstance(raw, dict):
        raise ScenarioError("SCENARIO_CONFIGURATION_INVALID")
    values = cast(dict[str, object], raw)
    if set(values) != set(ScenarioSpec.__dataclass_fields__):
        raise ScenarioError("SCENARIO_CONFIGURATION_INVALID")
    points = values["change_points"]
    if not isinstance(points, list):
        raise ScenarioError("SCENARIO_CONFIGURATION_INVALID")
    point_values = cast(list[object], points)
    if any(type(value) is not int for value in point_values):
        raise ScenarioError("SCENARIO_CONFIGURATION_INVALID")
    return ScenarioSpec(
        scenario_id=_string(values, "scenario_id"), version=_string(values, "version"),
        seed=_integer(values, "seed"), horizon=_integer(values, "horizon"),
        action_count=_integer(values, "action_count"), observation_width=_integer(values, "observation_width"),
        big_world_size=_integer(values, "big_world_size"), change_points=tuple(cast(list[int], point_values)),
    ), hashlib.sha256(payload).digest()


def _integer(values: dict[str, object], key: str) -> int:
    value = values[key]
    if type(value) is not int:
        raise ScenarioError("SCENARIO_CONFIGURATION_INVALID")
    return value


def _string(values: dict[str, object], key: str) -> str:
    value = values[key]
    if not isinstance(value, str):
        raise ScenarioError("SCENARIO_CONFIGURATION_INVALID")
    return value


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", type=Path, required=True)
    arguments = parser.parse_args()
    spec, identity = load_spec(arguments.spec)
    return serve(EnvironmentAdapter(spec, identity))


if __name__ == "__main__":
    raise SystemExit(main())
