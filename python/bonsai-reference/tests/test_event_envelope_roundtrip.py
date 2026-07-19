# pyright: reportUnknownMemberType=false, reportUnknownVariableType=false

from __future__ import annotations

import hashlib
import subprocess
from pathlib import Path

from google.protobuf import descriptor_pb2, descriptor_pool, message_factory

ROOT = Path(__file__).resolve().parents[3]


def run_contract_example(name: str, payload: bytes = b"") -> bytes:
    result = subprocess.run(
        ["cargo", "run", "--quiet", "-p", "bonsai-contracts", "--example", name],
        cwd=ROOT,
        input=payload,
        capture_output=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr.decode("utf-8", errors="replace")
    return result.stdout


def event_class() -> type:
    descriptor_set = descriptor_pb2.FileDescriptorSet()
    descriptor_set.ParseFromString(run_contract_example("event_descriptor"))
    pool = descriptor_pool.DescriptorPool()
    for file_descriptor in descriptor_set.file:
        pool.Add(file_descriptor)
    descriptor = pool.FindMessageTypeByName("bonsai.event.v1.EventEnvelope")
    return message_factory.GetMessageClass(descriptor)


def test_python_rust_python_round_trip_preserves_unknown_field() -> None:
    event_type = event_class()
    event = event_type()
    event.run_id = bytes([1]) * 16
    event.source_id = bytes([2]) * 16
    event.event_id = bytes([3]) * 16
    event.source_sequence = 7
    event.causal_parent_event_ids.append(bytes([4]) * 16)
    event.monotonic_time_ns = 10
    event.wall_time_unix_ns = 1_700_000_000_000_000_000
    event.event_type = "fixture.event/v1"
    event.payload_schema_epoch = 1
    event.payload_schema_minor = 0
    event.payload = b"fixture-payload"

    event.payload_sha256 = hashlib.sha256(event.payload).digest()
    event.availability = 1
    event.precision.representation = "bytes"

    encoded = event.SerializeToString(deterministic=True) + b"\x98\x06\x07"
    relayed = run_contract_example("event_roundtrip", encoded)
    assert relayed == encoded

    decoded = event_type()
    decoded.ParseFromString(relayed)
    assert decoded.run_id == event.run_id
    assert decoded.event_id == event.event_id
    assert decoded.payload == event.payload
