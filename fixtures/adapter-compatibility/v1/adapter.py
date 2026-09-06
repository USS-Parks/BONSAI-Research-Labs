"""Executable compatibility fixture; no scientific quality or isolation claim."""
from __future__ import annotations

import hashlib
import importlib.util
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "python/bonsai-reference/src"))
transport = importlib.import_module("bonsai_reference.transport")
read_frame, write_frame = transport.read_frame, transport.write_frame

minor = int(sys.argv[1])
binding = (Path(__file__).with_name("previous_adapter_pb2.py") if minor == 0
           else ROOT / "python/bonsai-reference/src/bonsai/adapter/v1/adapter_pb2.py")
spec = importlib.util.spec_from_file_location("compatibility_wire", binding)
assert spec is not None and spec.loader is not None
wire = importlib.util.module_from_spec(spec)
spec.loader.exec_module(wire)
caps = wire.CapabilityDeclaration(
    reset=True, work=False, feedback=False, asynchronous_events=False,
    accepted_input_types=["bonsai.fixture/v1"], retains_transitions=False,
    offline_updates=False, observer_data_access=False, privileged_state_access=False,
    filesystem_read=False, filesystem_write=True, network_access=False,
)
if minor == 1:
    caps.required_capabilities.extend(["bonsai.observation/v1", "bonsai.action/v1"])
    caps.optional_capabilities.append("vendor.diagnostic/v1")
fingerprint = hashlib.sha256(caps.SerializeToString(deterministic=True)).digest()
sequence = 0
state = "start"
while True:
    raw = read_frame(sys.stdin.buffer, 65536)
    if raw is None:
        break
    request = wire.AdapterFrame.FromString(raw)
    assert request.sequence == sequence and request.protocol_epoch == 1
    assert request.protocol_minor == minor
    kind = request.WhichOneof("message")
    response = wire.AdapterFrame(sequence=sequence, protocol_epoch=1, protocol_minor=minor,
                                 capability_fingerprint_sha256=fingerprint)
    if state == "start" and kind == "start":
        versions = request.start.accepted_versions
        assert (versions.minimum_epoch, versions.minimum_minor) <= (1, minor)
        assert (1, minor) <= (versions.maximum_epoch, versions.maximum_minor)
        response.handshake.CopyFrom(wire.Handshake(selected_epoch=1, selected_minor=minor, capabilities=caps))
        state = "configure"
    elif state == "configure" and kind == "configure":
        assert request.configure.accepted_capability_fingerprint_sha256 == fingerprint
        response.ack.operation = wire.OPERATION_CONFIGURE
        state = "reset"
    elif state in {"reset", "active"} and kind == "reset":
        response.ack.operation = wire.OPERATION_RESET
        state = "active"
    elif state == "active" and kind == "step":
        assert request.step.input_type == "bonsai.fixture/v1"
        assert hashlib.sha256(request.step.input).digest() == request.step.input_sha256
        action = hashlib.sha256(request.step.input).digest()[:1]
        response.step_result.CopyFrom(wire.StepResult(
            step_index=request.step.step_index, action=action,
            action_sha256=hashlib.sha256(action).digest()))
    elif kind == "stop":
        response.stopped.outcome_code = "STOPPED"
        state = "stopped"
    else:
        raise RuntimeError("COMPATIBILITY_FIXTURE_OUT_OF_ORDER")
    write_frame(sys.stdout.buffer, response.SerializeToString(deterministic=True), 65536)
    sequence += 1
    if state == "stopped":
        break
