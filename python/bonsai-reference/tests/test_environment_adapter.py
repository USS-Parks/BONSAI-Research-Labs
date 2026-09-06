from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import threading
import time
from collections.abc import Iterator, Mapping
from pathlib import Path
from typing import BinaryIO, Protocol, cast

import pytest
from bonsai.adapter.v1 import adapter_pb2 as wire
from bonsai_reference.adapter_wire import decode_into, encode, message_kind
from bonsai_reference.environment_adapter import ACT, MAXIMUM, OBSERVE, load_spec
from bonsai_reference.scenario import ScenarioSession
from bonsai_reference.transport import read_frame, write_frame

ROOT = Path(__file__).resolve().parents[3]
SPEC = ROOT / "fixtures/scenario/causal-v1/spec.json"


class LiveEnvironment:
    def __init__(self, configuration: Path = SPEC, arguments: list[str] | None = None) -> None:
        self.configuration = configuration
        environment = dict(os.environ)
        environment["PYTHONPATH"] = str(ROOT / "python/bonsai-reference/src")
        self.child = subprocess.Popen(
            [sys.executable, *(arguments if arguments is not None else
                               ["-m", "bonsai_reference.environment_adapter", "--spec", str(SPEC)])],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=environment,
        )
        self.watchdog = threading.Timer(8, self.child.kill)
        self.watchdog.start()
        self.sequence = 0
        self.query = 0
        self.fingerprint = b""
        self.transcript: list[dict[str, str]] = []

    def close(self) -> None:
        self.watchdog.cancel()
        if self.child.poll() is None:
            self.child.kill()
        self.child.wait(timeout=2)
        for pipe in [self.child.stdin, self.child.stdout, self.child.stderr]:
            if pipe is not None:
                pipe.close()

    def exchange(self, frame: wire.AdapterFrame) -> wire.AdapterFrame:
        frame.sequence = self.sequence
        frame.protocol_epoch = 1
        assert self.child.stdin is not None and self.child.stdout is not None
        started = time.monotonic()
        request = encode(frame)
        write_frame(cast(BinaryIO, self.child.stdin), request, MAXIMUM)
        payload = read_frame(cast(BinaryIO, self.child.stdout), MAXIMUM)
        assert payload is not None
        assert time.monotonic() - started < 5
        response = wire.AdapterFrame()
        decode_into(response, payload)
        self.transcript.extend([
            {"peer": "supervisor", "hex": request.hex()}, {"peer": "adapter", "hex": payload.hex()},
        ])
        assert response.sequence == self.sequence
        assert response.protocol_epoch == 1 and response.protocol_minor == 0
        if self.fingerprint:
            assert response.capability_fingerprint_sha256 == self.fingerprint
        self.sequence += 1
        return response

    def deadline(self) -> int:
        return (self.sequence + 1) * 1000

    def configure(self) -> None:
        response = self.exchange(wire.AdapterFrame(start=wire.Start(
            run_id=bytes([1] * 16), deterministic_seed=42, deadline_monotonic_ns=self.deadline(),
            accepted_versions=wire.VersionRange(minimum_epoch=1, maximum_epoch=1),
        )))
        assert message_kind(response) == "handshake"
        self.fingerprint = hashlib.sha256(encode(response.handshake.capabilities)).digest()
        assert response.capability_fingerprint_sha256 == self.fingerprint
        response = self.exchange(wire.AdapterFrame(configure=wire.Configure(
            configuration_sha256=hashlib.sha256(self.configuration.read_bytes()).digest(),
            accepted_capability_fingerprint_sha256=self.fingerprint, deadline_monotonic_ns=self.deadline(),
        )))
        assert response.ack.operation == wire.OPERATION_CONFIGURE

    def reset(self) -> None:
        response = self.exchange(wire.AdapterFrame(reset=wire.Reset(
            episode_id=bytes([2] * 16), deterministic_seed=42, deadline_monotonic_ns=self.deadline(),
        )))
        assert response.ack.operation == wire.OPERATION_RESET
        self.query = 0

    def step(self, input_type: str, payload: bytes, *, query: int | None = None) -> wire.AdapterFrame:
        response = self.exchange(wire.AdapterFrame(step=wire.Step(
            step_index=self.query if query is None else query, input_type=input_type, input=payload,
            input_sha256=hashlib.sha256(payload).digest(), deadline_monotonic_ns=self.deadline(),
        )))
        self.query += 1
        if message_kind(response) != "error":
            assert response.step_result.action_sha256 == hashlib.sha256(response.step_result.action).digest()
        return response

    def observe(self) -> wire.CausalObservation:
        response = self.step(OBSERVE, b"")
        assert message_kind(response) == "step_result"
        observation = wire.CausalObservation()
        decode_into(observation, response.step_result.action)
        return observation

    def stop(self) -> None:
        response = self.exchange(wire.AdapterFrame(stop=wire.Stop(
            reason_code="NORMAL_COMPLETION", deadline_monotonic_ns=self.deadline(),
        )))
        assert response.stopped.outcome_code == "STOPPED"
        assert self.child.wait(timeout=2) == 0


@pytest.fixture
def environment() -> Iterator[LiveEnvironment]:
    live = LiveEnvironment()
    try:
        yield live
    finally:
        live.close()


def positive_transcript(environment: LiveEnvironment) -> list[dict[str, str]]:
    environment.configure()
    environment.reset()
    observation = environment.observe()
    assert len(observation.observation) == 4
    assert list(observation.allowed_actions) == [0, 1, 2]
    for index in range(8):
        action = index % 3
        response = environment.step(ACT, encode(wire.CausalAction(step=index, action=action)))
        assert message_kind(response) == "step_result"
        transition = wire.CausalTransition()
        decode_into(transition, response.step_result.action)
        assert transition.step == index and transition.action == action
        assert transition.next.step == index + 1
        if transition.terminated or transition.truncated:
            assert not (transition.terminated and transition.truncated)
            assert list(transition.next.allowed_actions) == []
            break
    else:
        raise AssertionError("horizon must finish")
    environment.stop()
    return environment.transcript


def test_live_causal_protocol_matches_archived_transcript(environment: LiveEnvironment) -> None:
    actual = positive_transcript(environment)
    expected = json.loads((SPEC.parent / "wire-transcript.json").read_text(encoding="utf-8"))
    assert actual == expected


@pytest.mark.parametrize("case,code", [
    ("before_reset", "ADAPTER_MESSAGE_OUT_OF_ORDER"),
    ("before_observation", "ADAPTER_INPUT_TYPE_OR_ORDER_INVALID"),
    ("outer_step", "ADAPTER_STEP_OUT_OF_ORDER"),
    ("inner_step", "SCENARIO_STEP_OUT_OF_ORDER"),
    ("invalid_action", "SCENARIO_ACTION_INVALID"),
    ("bad_payload", "ADAPTER_PAYLOAD_INVALID"),
])
def test_live_invalid_requests_fail_bounded(environment: LiveEnvironment, case: str, code: str) -> None:
    environment.configure()
    if case != "before_reset":
        environment.reset()
    if case not in {"before_reset", "before_observation"}:
        environment.observe()
    payload = encode(wire.CausalAction(step=1 if case == "inner_step" else 0,
                                      action=3 if case == "invalid_action" else 0))
    if case == "bad_payload":
        payload = b"\xff"
    response = environment.step(ACT, payload, query=99 if case == "outer_step" else None)
    assert message_kind(response) == "error"
    assert response.error.reason_code == code
    assert len(encode(response)) < 256
    assert environment.child.wait(timeout=2) == 2


def test_live_post_truncation_action_is_rejected(environment: LiveEnvironment) -> None:
    environment.configure()
    environment.reset()
    environment.observe()
    next_step = 0
    transition = wire.CausalTransition()
    for index in range(8):
        result = environment.step(ACT, encode(wire.CausalAction(step=index, action=index % 3)))
        transition = wire.CausalTransition()
        decode_into(transition, result.step_result.action)
        next_step = transition.next.step
        if transition.terminated or transition.truncated:
            break
    assert transition.truncated and not transition.terminated
    rejected = environment.step(ACT, encode(wire.CausalAction(step=next_step, action=0)))
    assert rejected.error.reason_code == "SCENARIO_EPISODE_FINISHED"
    assert environment.child.wait(timeout=2) == 2


class _Descriptor(Protocol):
    fields_by_name: Mapping[str, object]


class _MessageType(Protocol):
    DESCRIPTOR: _Descriptor


def test_public_wire_types_have_no_diagnostic_or_future_schedule_fields() -> None:
    observation = cast(_MessageType, wire.CausalObservation).DESCRIPTOR
    transition = cast(_MessageType, wire.CausalTransition).DESCRIPTOR
    assert set(observation.fields_by_name) == {"stream_id", "step", "observation", "allowed_actions"}
    assert set(transition.fields_by_name) == {"step", "action", "reward", "next", "terminated", "truncated"}


def test_generated_bindings_match_the_locked_compiler() -> None:
    subprocess.run([sys.executable, str(ROOT / "scripts/check_python_protocol.py")], cwd=ROOT, check=True, timeout=30)


@pytest.mark.parametrize("case,code", [
    ("payload", "ADAPTER_PAYLOAD_INVALID"),
    ("oversized", "TRANSPORT_FRAME_TOO_LARGE"),
    ("partial", "TRANSPORT_HEADER_PARTIAL"),
])
def test_live_malformed_frames_return_bounded_errors(environment: LiveEnvironment, case: str, code: str) -> None:
    assert environment.child.stdin is not None and environment.child.stdout is not None
    if case == "payload":
        write_frame(cast(BinaryIO, environment.child.stdin), b"\xff", MAXIMUM)
    elif case == "oversized":
        environment.child.stdin.write((MAXIMUM + 1).to_bytes(4, "little"))
        environment.child.stdin.flush()
    else:
        environment.child.stdin.write(b"\x01")
        environment.child.stdin.close()
    payload = read_frame(cast(BinaryIO, environment.child.stdout), MAXIMUM)
    assert payload is not None and len(payload) < 256
    response = wire.AdapterFrame()
    decode_into(response, payload)
    assert response.error.reason_code == code
    assert environment.child.wait(timeout=2) == 2


def test_live_alternate_actions_change_outcomes_and_terminal_rejects_more_steps(environment: LiveEnvironment) -> None:
    alternate = LiveEnvironment()
    try:
        environment.configure()
        environment.reset()
        alternate.configure()
        alternate.reset()
        assert encode(environment.observe()) == encode(alternate.observe())
        # The test observer has the oracle; neither child response carries it.
        spec, _ = load_spec(SPEC)
        oracle = ScenarioSession(spec)
        oracle.reset(42)
        truth = oracle.diagnostic()
        winning = (truth.goal_state - truth.latent_state) % spec.big_world_size - 1
        first = environment.step(ACT, encode(wire.CausalAction(step=0, action=winning)))
        second = alternate.step(ACT, encode(wire.CausalAction(step=0, action=(winning + 1) % spec.action_count)))
        success, other = wire.CausalTransition(), wire.CausalTransition()
        decode_into(success, first.step_result.action)
        decode_into(other, second.step_result.action)
        assert success.terminated and success.reward == 1 and not success.truncated
        assert not other.terminated and other.reward == 0
        assert list(success.next.observation) != list(other.next.observation)
        rejected = environment.step(ACT, encode(wire.CausalAction(step=1, action=0)))
        assert rejected.error.reason_code == "SCENARIO_EPISODE_FINISHED"
        assert environment.child.wait(timeout=2) == 2
        alternate.stop()
    finally:
        alternate.close()
