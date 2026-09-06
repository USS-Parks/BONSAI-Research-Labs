"""Bounded synchronous client for the live diagnostic governor example."""
from __future__ import annotations

import json
import subprocess
import threading
from pathlib import Path
from typing import cast


class FeatureGovernor:
    """Own one external governor process; no decision is synthesized by the learner."""

    def __init__(self, executable: Path, *, work_limit: int = 1_000_000,
                 memory_limit: int = 262144, serialized_limit: int = 65536) -> None:
        self._failed = False
        self._closed = False
        self._child = subprocess.Popen(
            [str(executable), str(work_limit), str(memory_limit), str(serialized_limit)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
        )

    @property
    def pid(self) -> int:
        return self._child.pid

    @property
    def returncode(self) -> int | None:
        return self._child.poll()

    def work(self, request_id: str, amount: int) -> dict[str, object]:
        result = self._exchange({"kind": "work", "request_id": request_id, "amount": amount})
        if result.get("requested") != amount or result.get("work_class") != "feature_generation":
            self._failed = True
            raise ValueError("FEATURE_WORK_DECISION_MISMATCH")
        return result

    def allocation(self, request_id: str, allocated_bytes: int, serialized_bytes: int) -> dict[str, object]:
        result = self._exchange({"kind": "allocation", "request_id": request_id,
                                 "allocated_bytes": allocated_bytes, "serialized_bytes": serialized_bytes})
        if result.get("allocated_bytes") != allocated_bytes or result.get("serialized_bytes") != serialized_bytes:
            self._failed = True
            raise ValueError("FEATURE_ALLOCATION_DECISION_MISMATCH")
        return result

    def _exchange(self, request: dict[str, object]) -> dict[str, object]:
        try:
            return self._read_exchange(request)
        except Exception:
            self._failed = True
            raise

    def _read_exchange(self, request: dict[str, object]) -> dict[str, object]:
        if self._child.stdin is None or self._child.stdout is None:
            raise ValueError("FEATURE_GOVERNOR_PIPE_UNAVAILABLE")
        payload = json.dumps(request, separators=(",", ":")).encode() + b"\n"
        if len(payload) > 4096:
            raise ValueError("FEATURE_GOVERNOR_REQUEST_BOUND")
        self._child.stdin.write(payload)
        self._child.stdin.flush()
        response: list[bytes] = []
        output = self._child.stdout
        reader = threading.Thread(target=lambda: response.append(output.readline(4097)), daemon=True)
        reader.start()
        reader.join(timeout=10)
        if reader.is_alive():
            self._child.kill()
            self._child.wait(timeout=5)
            reader.join(timeout=5)
            raise ValueError("FEATURE_GOVERNOR_DEADLINE")
        if not response or not response[0].endswith(b"\n") or len(response[0]) > 4096:
            raise ValueError("FEATURE_GOVERNOR_RESPONSE_BOUND")
        decoded = cast(object, json.loads(response[0]))
        if type(decoded) is not dict:
            raise ValueError("FEATURE_GOVERNOR_RESPONSE_INVALID")
        result = cast(dict[str, object], decoded)
        if result.get("request_id") != request["request_id"] or result.get("outcome") not in {
            "admit", "defer", "reject"
        }:
            raise ValueError("FEATURE_GOVERNOR_RESPONSE_INVALID")
        return result

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        pipe_failed = False
        try:
            if self._child.stdin is not None:
                self._child.stdin.close()
        except OSError:
            pipe_failed = True
        try:
            code = self._child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self._child.kill()
            self._child.wait(timeout=5)
            if not self._failed:
                raise ValueError("FEATURE_GOVERNOR_SHUTDOWN_FAILED") from None
            return
        finally:
            if self._child.stdout is not None:
                self._child.stdout.close()
        if (code != 0 or pipe_failed) and not self._failed:
            raise ValueError("FEATURE_GOVERNOR_FAILED")
