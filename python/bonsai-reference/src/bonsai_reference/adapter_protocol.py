"""Shared ordered adapter handshake and lifecycle; payload semantics stay in adapters."""

from __future__ import annotations

import hashlib
import sys

from bonsai.adapter.v1 import adapter_pb2 as wire

from bonsai_reference.adapter_wire import decode_into, encode, message_kind
from bonsai_reference.scenario import ScenarioError
from bonsai_reference.transport import TransportError, read_frame, write_frame

MAXIMUM = 65_536


class OrderedAdapter:
    def __init__(self, declaration: wire.CapabilityDeclaration, configuration_sha256: bytes) -> None:
        self._capabilities = declaration
        self._configuration = configuration_sha256
        self._fingerprint = hashlib.sha256(encode(declaration)).digest()
        self._sequence = 0
        self._deadline = 0
        self._state = "created"

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
                selected_epoch=1, selected_minor=0, capabilities=self._capabilities,
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
            self._reset(frame.reset.deterministic_seed)
            self._state = "active"
            response = wire.AdapterFrame(ack=wire.Ack(operation=wire.OPERATION_RESET))
        elif self._state == "active" and kind == "step":
            response = self._step(frame.step)
        elif self._state == "active" and kind == "feedback":
            response = self._feedback(frame.feedback)
        elif self._state == "active" and kind == "work":
            response = self._work(frame.work)
        else:
            raise ScenarioError("ADAPTER_MESSAGE_OUT_OF_ORDER")
        self._deadline = max(
            frame.start.deadline_monotonic_ns, frame.configure.deadline_monotonic_ns,
            frame.reset.deadline_monotonic_ns, frame.step.deadline_monotonic_ns, frame.stop.deadline_monotonic_ns,
            frame.feedback.deadline_monotonic_ns, frame.work.deadline_monotonic_ns,
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

    def _check_deadline(self, deadline: int) -> None:
        if deadline <= self._deadline:
            raise ScenarioError("ADAPTER_DEADLINE_INVALID")


    def _reset(self, seed: int) -> None:
        raise ScenarioError("ADAPTER_RESET_UNSUPPORTED")

    def _step(self, request: wire.Step) -> wire.AdapterFrame:
        raise ScenarioError("ADAPTER_MESSAGE_OUT_OF_ORDER")

    def _feedback(self, request: wire.Feedback) -> wire.AdapterFrame:
        raise ScenarioError("ADAPTER_MESSAGE_OUT_OF_ORDER")

    def _work(self, request: wire.Work) -> wire.AdapterFrame:
        raise ScenarioError("ADAPTER_MESSAGE_OUT_OF_ORDER")


def serve(adapter: OrderedAdapter) -> int:
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
