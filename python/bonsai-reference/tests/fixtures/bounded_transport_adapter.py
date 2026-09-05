"""Linux BX-02 fault fixtures; every stalled process has an eight-second lifetime."""

from __future__ import annotations

import json
import os
import struct
import subprocess
import sys
import time


def frame(payload: bytes) -> None:
    sys.stdout.buffer.write(struct.pack("<I", len(payload)) + payload)
    sys.stdout.buffer.flush()


def main() -> None:
    mode = sys.argv[1]
    descendant = None
    if mode in {"grandchild", "exited_parent"}:
        descendant = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(8)"])
    frame(json.dumps([os.getpid()] + ([] if descendant is None else [descendant.pid])).encode())
    if mode in {"normal", "exited_parent"}:
        return
    if mode == "partial_hang":
        sys.stdout.buffer.write(struct.pack("<I", 8) + b"xx")
        sys.stdout.buffer.flush()
    if mode == "stderr_flood":
        until = time.monotonic() + 8
        while time.monotonic() < until:
            sys.stderr.buffer.write(b"x" * 4096)
            sys.stderr.buffer.flush()
        return
    time.sleep(8)
    if descendant is not None:
        descendant.wait(timeout=1)


if __name__ == "__main__":
    main()
