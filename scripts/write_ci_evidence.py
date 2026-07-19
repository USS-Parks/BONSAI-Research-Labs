"""Write a sanitized hosted-CI classification artifact."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser()
    result.add_argument("--output", type=Path, required=True)
    result.add_argument("--source-revision", required=True)
    result.add_argument("--runner-label", required=True)
    result.add_argument("--runner-os", required=True)
    result.add_argument("--runner-arch", required=True)
    result.add_argument("--run-id", required=True)
    result.add_argument("--run-attempt", required=True)
    result.add_argument("--job", required=True)
    return result


def main() -> int:
    args = parser().parse_args()
    record = {
        "schema": "bonsai.ci-evidence/v1",
        "evidence_class": "hosted-ci",
        "runner_class": "github-hosted-ephemeral-vm",
        "runner_label": args.runner_label,
        "runner_os": args.runner_os,
        "runner_arch": args.runner_arch,
        "source_revision": args.source_revision,
        "workflow_run_id": args.run_id,
        "workflow_run_attempt": args.run_attempt,
        "workflow_job": args.job,
        "physical_acceptance": False,
        "energy_claim": False,
        "long_duration_claim": False,
    }
    payload = (json.dumps(record, indent=2, sort_keys=True) + "\n").encode()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(payload)
    print(f"wrote {args.output.as_posix()} sha256={hashlib.sha256(payload).hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
