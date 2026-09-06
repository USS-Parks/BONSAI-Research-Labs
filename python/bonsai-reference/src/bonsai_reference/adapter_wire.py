"""Typed access to generated Protobuf bindings and bounded adapter framing."""

from __future__ import annotations

from typing import Protocol, cast

from bonsai.adapter.v1 import adapter_pb2 as wire

from bonsai_reference.scenario import ScenarioError


class _Message(Protocol):
    def SerializeToString(self, *, deterministic: bool = False) -> bytes: ...
    def ParseFromString(self, data: bytes) -> int: ...
    def WhichOneof(self, group: str) -> str | None: ...


def encode(message: object) -> bytes:
    """Serialize generated messages deterministically across Python and Rust."""
    return cast(_Message, message).SerializeToString(deterministic=True)


def decode_into(message: object, payload: bytes) -> None:
    """Classify decoder errors without echoing arbitrary adapter bytes."""
    try:
        cast(_Message, message).ParseFromString(payload)
    except Exception as error:
        raise ScenarioError("ADAPTER_PAYLOAD_INVALID") from error


def message_kind(frame: wire.AdapterFrame) -> str | None:
    return cast(_Message, frame).WhichOneof("message")
