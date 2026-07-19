"""Reconcile the BONSAI M0 governed-foundation checkpoint."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PSPR = ROOT / "BONSAI Research Charter" / "BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md"
EXPECTED_ORIGIN = "https://github.com/USS-Parks/BONSAI-Research-Labs.git"
REQUIRED_FILES = (
    "README.md",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "pyproject.toml",
    "uv.lock",
    "docs/governance/SOURCE-OF-TRUTH.md",
    "docs/governance/PUBLICATION-POLICY.md",
    "docs/governance/RISK-AND-BLOCKER-REGISTER.md",
    "docs/governance/PARKED-SCOPE-LEDGER.md",
    "docs/governance/CLAIM-TO-EVIDENCE-MATRIX.md",
    "docs/governance/TERMINOLOGY-AND-UNITS.md",
    "docs/sessions/BONSAI-DEVLOG.md",
    "docs/verification/BONSAI-VERIFICATION-LOG.md",
    "docs/verification/TEST-MATRIX.md",
    "docs/verification/M0-CHECKPOINT.md",
    "schemas/registry/terminology-v1.json",
    ".github/workflows/ci.yml",
)


def git(*args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    return result.stdout.strip()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--allow-dirty", action="store_true", help="allow the pre-commit BG-10 construction state")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    errors: list[str] = []
    for relative in REQUIRED_FILES:
        if not (ROOT / relative).is_file():
            errors.append(f"missing required M0 file: {relative}")

    psp_text = PSPR.read_text(encoding="utf-8")
    for number in range(1, 10):
        if not re.search(rf"^- \[x] \*\*BG-{number:02d} —", psp_text, re.MULTILINE):
            errors.append(f"BG-{number:02d} is not complete in the PSPR")
    bg10_status = re.search(r"^- \[([x~])] \*\*BG-10 —", psp_text, re.MULTILINE)
    if not bg10_status:
        errors.append("BG-10 must be executing or complete")
    if re.search(r"^- \[x] \*\*(?:BC|BR|BM|BQ|BK|BE|BV)-", psp_text, re.MULTILINE):
        errors.append("a post-M0 prompt is incorrectly marked complete")

    origin = git("remote", "get-url", "origin")
    if origin != EXPECTED_ORIGIN:
        errors.append(f"origin mismatch: {origin}")
    if git("rev-parse", "--show-toplevel").replace("\\", "/") != ROOT.as_posix():
        errors.append("Git top level differs from the active isolated worktree")
    dirty = git("status", "--porcelain")
    if dirty and not args.allow_dirty:
        errors.append("working tree is not clean")

    root_readme = (ROOT / "README.md").read_text(encoding="utf-8")
    if "Implementation claims: none" not in root_readme or "BONSAI-Research-Labs" not in root_readme:
        errors.append("root README lacks the no-claim boundary or authoritative repository")
    claim_matrix = (ROOT / "docs/governance/CLAIM-TO-EVIDENCE-MATRIX.md").read_text(encoding="utf-8")
    if claim_matrix.count("| not-run |") < 7 or "M0 has no implementation claim" not in claim_matrix:
        errors.append("claim matrix does not keep M0 and C0-C5 at not-run")
    parked = (ROOT / "docs/governance/PARKED-SCOPE-LEDGER.md").read_text(encoding="utf-8")
    if len(re.findall(r"^\| P-\d{2} \|", parked, re.MULTILINE)) != 9:
        errors.append("parked-scope ledger does not contain nine seeded items")
    checkpoint = (ROOT / "docs/verification/M0-CHECKPOINT.md").read_text(encoding="utf-8")
    has_scope_stop = "not authorized by `run M0 STS`" in checkpoint
    has_no_claim = "does not establish a working BONSAI instrument" in checkpoint
    if not has_scope_stop or not has_no_claim:
        errors.append("M0 checkpoint lacks the BC-01 stop or no-implementation-claim boundary")

    if errors:
        print("M0 checkpoint failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    state = "dirty construction allowed" if dirty else "clean"
    print(f"M0 checkpoint passed at {git('rev-parse', 'HEAD')}: {state}; BG-01..BG-09 complete; BC-01 not authorized")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
