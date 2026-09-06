"""Verify checked-in Python bindings with Cargo's already locked protoc compiler."""

from __future__ import annotations

import os
import platform
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def compiler() -> Path:
    architecture = {"arm64": "aarch_64", "aarch64": "aarch_64", "amd64": "x86_64"}.get(
        platform.machine().lower(), platform.machine().lower(),
    )
    system = {"win32": "win32", "darwin": "macos", "linux": "linux"}.get(sys.platform)
    if system is None:
        raise RuntimeError("PYTHON_PROTOCOL_COMPILER_PLATFORM_UNSUPPORTED")
    package = "protoc-bin-vendored-win32" if system == "win32" else f"protoc-bin-vendored-{system}-{architecture}"
    locked_identity = f'name = "{package}"\nversion = "3.2.0"'
    if locked_identity not in (ROOT / "Cargo.lock").read_text(encoding="utf-8"):
        raise RuntimeError("PYTHON_PROTOCOL_COMPILER_LOCK_MISMATCH")
    registry = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo"))) / "registry/src"
    executable = "protoc.exe" if system == "win32" else "protoc"
    # This exact package version is already pinned by the workspace Cargo.lock.
    candidates = sorted(registry.glob(f"*/{package}-3.2.0/bin/{executable}"))
    if not candidates:
        raise RuntimeError("PYTHON_PROTOCOL_LOCKED_COMPILER_NOT_CACHED: build the Cargo workspace first")
    return candidates[0]


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="bx04-protobuf-", dir=ROOT / "target") as directory:
        subprocess.run([
            str(compiler()), "--proto_path=proto", f"--python_out={directory}", f"--pyi_out={directory}",
            "proto/bonsai/adapter/v1/adapter.proto",
        ], cwd=ROOT, check=True, timeout=20)
        for filename in ["adapter_pb2.py", "adapter_pb2.pyi"]:
            generated = Path(directory) / "bonsai/adapter/v1" / filename
            checked_in = ROOT / "python/bonsai-reference/src/bonsai/adapter/v1" / filename
            if generated.read_bytes() != checked_in.read_bytes():
                raise RuntimeError(f"PYTHON_PROTOCOL_GENERATED_MISMATCH: {filename}")
    print("Python protocol bindings match locked protoc 3.2.0 package output")


if __name__ == "__main__":
    main()
