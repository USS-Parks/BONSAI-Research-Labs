"""Online causal environment on the existing framed Protobuf process protocol."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import cast

from bonsai.adapter.v1 import adapter_pb2 as wire

from bonsai_reference.adapter_wire import decode_into, encode, message_kind
from bonsai_reference.scenario import ScenarioError, ScenarioSession, ScenarioSpec, SessionObservation
from bonsai_reference.transport import TransportError, read_frame, write_frame

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


class EnvironmentAdapter:
    def __init__(self, spec: ScenarioSpec, configuration_sha256: bytes) -> None:
        self.session = ScenarioSession(spec)
        self._configuration = configuration_sha256
        self._fingerprint = hashlib.sha256(encode(capabilities())).digest()
        self._sequence = 0
        self._deadline = 0
        self._state = "created"
        self._query = 0
        self._observed = False

    def accept(self, frame: wire.AdapterFrame) -> wire.AdapterFrame:
        """One ordered request produces one bounded response or stable failure."""
        if frame.sequence != self._sequence:
            raise ScenarioError("ADAPTER_SEQUENCE_INVALID")
        if frame.protocol_epoch != 1 or frame.protocol_minor != 0:
            raise ScenarioError("ADAPTER_VERSION_INVALID")
        kind = message_kind(frame)
        if kind == "stop" and self._state != "stopped":
            self._check_deadline(frame.stop.deadline_monotonic_ns)
            if not frame.stop.reason_code:
                raise ScenarioError("ADAPTER_STOP_INVALID")
            self._state = "stopped"
            response = wire.AdapterFrame(stopped=wire.Stopped(outcome_code="STOPPED"))
        elif self._state == "created" and kind == "start":
            start = frame.start
            versions = start.accepted_versions
            minimum = (versions.minimum_epoch, versions.minimum_minor)
            maximum = (versions.maximum_epoch, versions.maximum_minor)
            if len(start.run_id) != 16 or not any(start.run_id):
                raise ScenarioError("ADAPTER_RUN_ID_INVALID")
            if minimum[0] == 0 or not minimum <= (1, 0) <= maximum:
                raise ScenarioError("ADAPTER_VERSION_INVALID")
            self._check_deadline(start.deadline_monotonic_ns)
            self._state = "configure"
            response = wire.AdapterFrame(handshake=wire.Handshake(
                selected_epoch=1, selected_minor=0, capabilities=capabilities(),
            ))
        elif self._state == "configure" and kind == "configure":
            config = frame.configure
            if config.configuration_sha256 != self._configuration:
                raise ScenarioError("ADAPTER_CONFIGURATION_MISMATCH")
            if config.accepted_capability_fingerprint_sha256 != self._fingerprint:
                raise ScenarioError("ADAPTER_CAPABILITY_MISMATCH")
            self._check_deadline(config.deadline_monotonic_ns)
            self._state = "ready"
            response = wire.AdapterFrame(ack=wire.Ack(operation=wire.OPERATION_CONFIGURE))
        elif self._state in {"ready", "active"} and kind == "reset":
            if len(frame.reset.episode_id) != 16 or not any(frame.reset.episode_id):
                raise ScenarioError("ADAPTER_EPISODE_ID_INVALID")
            self._check_deadline(frame.reset.deadline_monotonic_ns)
            self.session.reset(frame.reset.deterministic_seed)
            self._state, self._query, self._observed = "active", 0, False
            response = wire.AdapterFrame(ack=wire.Ack(operation=wire.OPERATION_RESET))
        elif self._state == "active" and kind == "step":
            response = self._step(frame.step)
        else:
            raise ScenarioError("ADAPTER_MESSAGE_OUT_OF_ORDER")
        self._deadline = max(
            frame.start.deadline_monotonic_ns, frame.configure.deadline_monotonic_ns,
            frame.reset.deadline_monotonic_ns, frame.step.deadline_monotonic_ns, frame.stop.deadline_monotonic_ns,
        )
        response.sequence = self._sequence
        response.protocol_epoch = 1
        response.protocol_minor = 0
        response.capability_fingerprint_sha256 = self._fingerprint
        self._sequence += 1
        return response

    def failure(self, code: str) -> wire.AdapterFrame:
        """Fatal responses retain the same sequence and capability identity."""
        return wire.AdapterFrame(
            sequence=self._sequence, protocol_epoch=1, capability_fingerprint_sha256=self._fingerprint,
            error=wire.ProtocolError(reason_code=code),
        )

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

    def _check_deadline(self, deadline: int) -> None:
        if deadline <= self._deadline:
            raise ScenarioError("ADAPTER_DEADLINE_INVALID")


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
    adapter = EnvironmentAdapter(spec, identity)
    while True:
        try:
            payload = read_frame(sys.stdin.buffer, MAXIMUM)
            if payload is None:
                return 0
            frame = wire.AdapterFrame()
            decode_into(frame, payload)
            response = adapter.accept(frame)
        except (ScenarioError, TransportError) as error:
            response = adapter.failure(error.code)
            write_frame(sys.stdout.buffer, encode(response), MAXIMUM)
            return 2
        write_frame(sys.stdout.buffer, encode(response), MAXIMUM)
        if message_kind(response) == "stopped":
            return 0


if __name__ == "__main__":
    raise SystemExit(main())
