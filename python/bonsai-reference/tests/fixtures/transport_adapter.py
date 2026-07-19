"""Cross-language transport fixture process."""

from __future__ import annotations

import struct
import sys
import time
from pathlib import Path

SOURCE = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SOURCE))

from bonsai_reference.transport import read_frame, write_frame  # noqa: E402

MAXIMUM = 1024


def main() -> int:
    mode = sys.argv[1]
    if mode == "echo":
        sys.stderr.buffer.write(b"fixture-stderr\n")
        sys.stderr.buffer.flush()
        while (frame := read_frame(sys.stdin.buffer, MAXIMUM)) is not None:
            write_frame(sys.stdout.buffer, frame, MAXIMUM)
        return 0
    if mode == "partial":
        sys.stdout.buffer.write(struct.pack("<I", 8) + b"xx")
        sys.stdout.buffer.flush()
        return 0
    if mode == "oversized":
        sys.stdout.buffer.write(struct.pack("<I", MAXIMUM + 1))
        sys.stdout.buffer.flush()
        return 0
    if mode == "stalled":
        time.sleep(5)
        return 0
    if mode == "flood":
        sys.stderr.buffer.write(b"flood-ready\n")
        sys.stderr.buffer.flush()
        for _ in range(10_000):
            write_frame(sys.stdout.buffer, b"x" * 512, MAXIMUM)
        return 0
    raise ValueError(f"unknown mode: {mode}")


if __name__ == "__main__":
    raise SystemExit(main())
