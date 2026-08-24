"""Rebuild required targets without network after locks or a vendor tree exist."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import UTC, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evidence" / "release-candidate" / "offline-restore.json"


def run(command: list[str], extra_env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    if extra_env:
        env.update(extra_env)
    return subprocess.run(command, cwd=ROOT, check=False, text=True, encoding="utf-8", env=env, capture_output=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--vendor", action="store_true", help="write vendor/ from Cargo.lock")
    parser.add_argument("--verify", action="store_true", help="run cargo --offline and frozen pytest")
    args = parser.parse_args()

    if not (ROOT / "Cargo.lock").is_file() or not (ROOT / "uv.lock").is_file():
        print("offline_restore: committed lockfiles are required", file=sys.stderr)
        return 1

    if args.vendor:
        vendor = run(["cargo", "vendor", "--versioned-dirs", "vendor"])
        if vendor.returncode != 0:
            print(vendor.stderr, file=sys.stderr)
            return vendor.returncode

    rustc = run(["rustc", "-vV"])
    python = run([sys.executable, "-V"])
    record: dict[str, object] = {
        "schema": "bonsai.offline-restore/v1",
        "evidence_class": "hosted-ci-or-local",
        "physical_clean_machine": False,
        "recorded_at": datetime.now(UTC).isoformat(),
        "rustc": rustc.stdout.strip(),
        "python": python.stdout.strip(),
        "vendor_present": (ROOT / "vendor").is_dir(),
        "reproducibility_delta": "normalized lock/SBOM hashes only; bit-for-bit artifacts are not claimed",
        "physical_clean_machine_status": "not-run",
    }

    if args.verify:
        cargo = run(
            ["cargo", "test", "--offline", "--workspace", "--all-features"],
            extra_env={"CARGO_NET_OFFLINE": "true"},
        )
        uv = run(["uv", "run", "--frozen", "pytest"])
        record["cargo_offline_exit"] = cargo.returncode
        record["pytest_frozen_exit"] = uv.returncode
        if cargo.returncode != 0:
            sys.stderr.write(cargo.stdout)
            sys.stderr.write(cargo.stderr)
            OUT.parent.mkdir(parents=True, exist_ok=True)
            OUT.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
            return cargo.returncode
        if uv.returncode != 0:
            sys.stderr.write(uv.stdout)
            sys.stderr.write(uv.stderr)
            OUT.parent.mkdir(parents=True, exist_ok=True)
            OUT.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
            return uv.returncode

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print("offline_restore: lockfiles present; physical clean-machine remains not-run")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
