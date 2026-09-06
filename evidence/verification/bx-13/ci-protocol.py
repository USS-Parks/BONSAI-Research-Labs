"""Run BX-13 protocol tests and guard verifier source bytes across hosted OSes."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import re
import subprocess
import sys
import time
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[3]
REFERENCE = ROOT / "crates/bonsai-bundle/src/governed/reference.rs"


def git(*arguments: str, binary: bool = False) -> bytes | str:
    result = subprocess.check_output(["git", *arguments], cwd=ROOT, timeout=120)
    return result if binary else result.decode().strip()


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def verifier_source_paths() -> tuple[str, ...]:
    text = REFERENCE.read_text(encoding="utf-8")
    _, marker, remainder = text.partition("const SOURCES")
    block, closing, _ = remainder.partition("\n];")
    assert marker and closing, "BX13_VERIFIER_SOURCES_NOT_FOUND"
    paths = tuple(re.findall(r'^\s*"([^"]+)",\s*$', block, flags=re.MULTILINE))
    include_count = len(re.findall(r"\binclude_bytes!\s*\(", block))
    assert paths and len(paths) == include_count, "BX13_VERIFIER_SOURCE_COUNT_MISMATCH"
    assert len(paths) == len(set(paths)), "BX13_VERIFIER_SOURCE_DUPLICATE"
    for value in paths:
        relative = PurePosixPath(value)
        assert not relative.is_absolute() and ".." not in relative.parts, "BX13_VERIFIER_SOURCE_PATH_INVALID"
        assert (ROOT / relative).is_file(), f"BX13_VERIFIER_SOURCE_MISSING:{value}"
    return paths


def git_attributes(path: str) -> dict[str, str]:
    output = git("check-attr", "text", "eol", "--", path)
    assert isinstance(output, str)
    attributes: dict[str, str] = {}
    for line in output.splitlines():
        reported, name, value = line.rsplit(": ", 2)
        assert reported == path and name in {"text", "eol"}, f"BX13_GIT_ATTRIBUTE_OUTPUT_INVALID:{path}"
        attributes[name] = value
    assert set(attributes) == {"text", "eol"}, f"BX13_GIT_ATTRIBUTE_OUTPUT_INCOMPLETE:{path}"
    return attributes


def source_record(
    path: str,
    git_blob: bytes,
    filtered_blob: bytes,
    checkout: bytes,
    attributes: dict[str, str],
    *,
    enforce_checkout: bool,
) -> tuple[dict[str, object], list[str]]:
    violations = []
    if attributes.get("text") != "set":
        violations.append(f"{path}:text={attributes.get('text')}")
    if attributes.get("eol") != "lf":
        violations.append(f"{path}:eol={attributes.get('eol')}")
    if enforce_checkout and git_blob != filtered_blob:
        violations.append(f"{path}:filtered-bytes-differ")
    if enforce_checkout and git_blob != checkout:
        violations.append(f"{path}:checkout-bytes-differ")
    record = {
        "attributes": attributes,
        "git_blob": {"bytes": len(git_blob), "sha256": digest(git_blob)},
        "filtered_blob": {"bytes": len(filtered_blob), "sha256": digest(filtered_blob)},
        "checkout": {"bytes": len(checkout), "sha256": digest(checkout)},
        "filtered_matches_git_blob": filtered_blob == git_blob,
        "checkout_matches_git_blob": checkout == git_blob,
    }
    return record, violations


def verifier_source_identity(*, enforce_checkout: bool) -> tuple[dict[str, object], list[str]]:
    records: dict[str, object] = {}
    violations: list[str] = []
    for path in verifier_source_paths():
        git_blob = git("cat-file", "blob", f"HEAD:{path}", binary=True)
        filtered_blob = git("cat-file", "--filters", f"HEAD:{path}", binary=True)
        assert isinstance(git_blob, bytes) and isinstance(filtered_blob, bytes)
        record, problems = source_record(
            path,
            git_blob,
            filtered_blob,
            (ROOT / PurePosixPath(path)).read_bytes(),
            git_attributes(path),
            enforce_checkout=enforce_checkout,
        )
        records[path] = record
        violations.extend(problems)
    return records, violations


def main() -> int:
    assert not sys.argv[1:], "BX13_CI_PROTOCOL_ARGUMENT_INVALID"
    batch = ROOT / "target" / f"bx13-ci-{time.time_ns()}"
    batch.mkdir(parents=True)
    command = [
        sys.executable,
        "-B",
        "-m",
        "pytest",
        "python/bonsai-reference/tests/test_option_adapter.py",
        "--basetemp",
        str(batch / "pytest-tmp"),
        "-o",
        f"cache_dir={batch / 'pytest-cache'}",
        f"--junitxml={batch / 'pytest.xml'}",
        "-ra",
    ]
    started = time.monotonic_ns()
    try:
        completed = subprocess.run(command, cwd=ROOT, capture_output=True, check=False, timeout=600)
        stdout, stderr, exit_code = completed.stdout, completed.stderr, completed.returncode
    except subprocess.TimeoutExpired as error:
        stdout = error.stdout or b""
        stderr = (error.stderr or b"") + b"\nBX13_CI_PROTOCOL_TIMEOUT\n"
        exit_code = 124
    elapsed = time.monotonic_ns() - started
    (batch / "pytest.stdout.txt").write_bytes(stdout)
    (batch / "pytest.stderr.txt").write_bytes(stderr)

    patch = git("diff", "HEAD", "--binary", binary=True)
    revision = git("rev-parse", "HEAD")
    tree = git("rev-parse", "HEAD^{tree}")
    assert isinstance(patch, bytes) and isinstance(revision, str) and isinstance(tree, str)
    hosted = os.getenv("GITHUB_ACTIONS") == "true"
    records, violations = verifier_source_identity(enforce_checkout=hosted)
    github_sha = os.getenv("GITHUB_SHA")
    if hosted and patch:
        violations.append("checkout:tracked-patch-present")
    if hosted and github_sha != revision:
        violations.append("checkout:github-sha-mismatch")
    source_identity = {
        "revision": revision,
        "tree": tree,
        "github_sha": github_sha,
        "tracked_patch_bytes": len(patch),
        "tracked_patch_sha256": digest(patch),
        "workflow_sha256": digest((ROOT / ".github/workflows/ci.yml").read_bytes()),
        "test_sha256": digest((ROOT / "python/bonsai-reference/tests/test_option_adapter.py").read_bytes()),
        "wrapper_sha256": digest(Path(__file__).read_bytes()),
        "verifier_source_count": len(records),
        "verifier_sources": records,
        "source_guard": "pass" if not violations else "fail",
        "source_guard_violations": violations,
    }
    (batch / "source-identity.json").write_text(json.dumps(source_identity, indent=2) + "\n", encoding="utf-8")
    if violations:
        (batch / "source-guard-error.txt").write_text("\n".join(violations) + "\n", encoding="utf-8")
        if exit_code == 0:
            exit_code = 1

    host = {
        "system": platform.system(),
        "release": platform.release(),
        "machine": platform.machine(),
        "python_version": platform.python_version(),
        "python_executable_sha256": digest(Path(sys.executable).read_bytes()),
        "github_actions": hosted,
        "runner_os": os.getenv("RUNNER_OS"),
        "runner_arch": os.getenv("RUNNER_ARCH"),
        "runner_name": os.getenv("RUNNER_NAME"),
        "matrix_os_family": os.getenv("BONSAI_CI_OS_FAMILY"),
        "matrix_arch": os.getenv("BONSAI_CI_ARCH"),
        "matrix_runner": os.getenv("BONSAI_CI_RUNNER"),
        "wsl": "microsoft" in platform.release().lower(),
        "hosted_ci_protocol_evidence": hosted,
        "os_resource_controller_evidence": False,
        "wsl_governed_matrix_evidence": False,
        "physical_host_acceptance": False,
    }
    summary = {
        "schema": "bonsai.online-option-ci-protocol/v1",
        "test_file": "python/bonsai-reference/tests/test_option_adapter.py",
        "command": command,
        "exit_code": exit_code,
        "elapsed_ns": elapsed,
        "stdout": {"bytes": len(stdout), "sha256": digest(stdout)},
        "stderr": {"bytes": len(stderr), "sha256": digest(stderr)},
        "source_identity": source_identity,
        "host": host,
    }
    (batch / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"batch": batch.relative_to(ROOT).as_posix(), "exit_code": exit_code}), flush=True)
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
