"""Bounded binary framing for BONSAI child-process adapters."""

from __future__ import annotations

from typing import BinaryIO, Final

HARD_MAX_TRANSPORT_FRAME_BYTES: Final = 16 * 1024 * 1024


class TransportError(Exception):
    """Stable framing failure with a machine-oriented code."""

    def __init__(self, code: str) -> None:
        super().__init__(code)
        self.code = code


def read_frame(stream: BinaryIO, maximum: int) -> bytes | None:
    """Read one bounded little-endian length-prefixed frame."""
    _validate_maximum(maximum)
    first = stream.read(1)
    if first == b"":
        return None
    header = first + _read_exact(stream, 3, "TRANSPORT_HEADER_PARTIAL")
    declared = int.from_bytes(header, "little")
    if declared == 0:
        raise TransportError("TRANSPORT_EMPTY_FRAME")
    if declared > maximum:
        raise TransportError("TRANSPORT_FRAME_TOO_LARGE")
    return _read_exact(stream, declared, "TRANSPORT_PAYLOAD_PARTIAL")


def write_frame(stream: BinaryIO, payload: bytes, maximum: int) -> None:
    """Write and flush one bounded little-endian length-prefixed frame."""
    _validate_maximum(maximum)
    if not payload:
        raise TransportError("TRANSPORT_EMPTY_FRAME")
    if len(payload) > maximum:
        raise TransportError("TRANSPORT_FRAME_TOO_LARGE")
    stream.write(len(payload).to_bytes(4, "little"))
    stream.write(payload)
    stream.flush()


def _validate_maximum(maximum: int) -> None:
    if not 0 < maximum <= HARD_MAX_TRANSPORT_FRAME_BYTES:
        raise TransportError("TRANSPORT_LIMIT_INVALID")


def _read_exact(stream: BinaryIO, length: int, partial_code: str) -> bytes:
    chunks = bytearray()
    while len(chunks) < length:
        chunk = stream.read(length - len(chunks))
        if chunk == b"":
            raise TransportError(partial_code)
        chunks.extend(chunk)
    return bytes(chunks)
