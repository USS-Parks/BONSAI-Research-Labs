from __future__ import annotations

import io

from bonsai_reference.transport import TransportError, read_frame, write_frame


def test_frame_round_trip_and_exact_failure_codes() -> None:
    stream = io.BytesIO()
    write_frame(stream, b"payload", 32)
    stream.seek(0)
    assert read_frame(stream, 32) == b"payload"

    assert_failure(b"\x01\x00", "TRANSPORT_HEADER_PARTIAL")
    assert_failure(b"\x04\x00\x00\x00xx", "TRANSPORT_PAYLOAD_PARTIAL")
    assert_failure(b"\x21\x00\x00\x00", "TRANSPORT_FRAME_TOO_LARGE")


def assert_failure(encoded: bytes, expected: str) -> None:
    try:
        read_frame(io.BytesIO(encoded), 32)
    except TransportError as error:
        assert error.code == expected
    else:
        raise AssertionError(f"expected {expected}")
