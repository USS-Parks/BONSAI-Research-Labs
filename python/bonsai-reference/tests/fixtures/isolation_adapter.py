"""Inspect only the arguments, environment, protocol, and filesystem roots granted to an adapter."""

from __future__ import annotations

import json
import os
import struct
import sys
from collections.abc import Iterable
from pathlib import Path
from typing import TypedDict

MAXIMUM = 1024 * 1024


class InspectionResult(TypedDict):
    arguments: list[str]
    current_directory: str
    environment_keys: list[str]
    granted_inputs: dict[str, str]
    discovered_files: list[str]
    observer_canary_discovered: bool
    work_probe_written: bool


def write_frame(payload: bytes) -> None:
    if not payload or len(payload) > MAXIMUM:
        raise ValueError("invalid inspection frame")
    sys.stdout.buffer.write(struct.pack("<I", len(payload)) + payload)
    sys.stdout.buffer.flush()


def candidate_roots(arguments: list[str]) -> set[Path]:
    roots = {Path.cwd()}
    for key in ("BONSAI_AGENT_ROOT", "BONSAI_INPUT_ROOT", "BONSAI_WORK_ROOT"):
        roots.add(Path(os.environ[key]))
    for index, value in enumerate(arguments):
        if value == "--bonsai-input":
            roots.add(Path(arguments[index + 1].split("=", maxsplit=1)[1]).parent)
        elif value == "--bonsai-work-dir":
            roots.add(Path(arguments[index + 1]))
    return roots


def discover_files(roots: Iterable[Path]) -> list[str]:
    discovered: list[str] = []
    for root in sorted(roots):
        for directory, child_directories, files in os.walk(root, followlinks=False):
            child_directories.sort()
            discovered.extend(str(Path(directory) / name) for name in sorted(files))
    return discovered


def main() -> int:
    arguments = sys.argv[1:]
    roots = candidate_roots(arguments)
    discovered = discover_files(roots)
    granted_inputs: dict[str, str] = {}
    for index, value in enumerate(arguments):
        if value == "--bonsai-input":
            name, path = arguments[index + 1].split("=", maxsplit=1)
            granted_inputs[name] = Path(path).read_text(encoding="utf-8")
    work_root = Path(os.environ["BONSAI_WORK_ROOT"])
    work_probe = work_root / "inspection-write.txt"
    work_probe.write_text("agent-owned", encoding="utf-8")
    result: InspectionResult = {
        "arguments": arguments,
        "current_directory": str(Path.cwd()),
        "environment_keys": sorted(os.environ),
        "granted_inputs": granted_inputs,
        "discovered_files": discovered,
        "observer_canary_discovered": any(Path(path).name == "observer-canary.txt" for path in discovered),
        "work_probe_written": work_probe.read_text(encoding="utf-8") == "agent-owned",
    }
    write_frame(json.dumps(result, sort_keys=True, separators=(",", ":")).encode())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
