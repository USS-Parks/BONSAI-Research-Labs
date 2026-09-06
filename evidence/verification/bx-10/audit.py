"""Audit the generic authority boundary and preserved source identities."""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
def main() -> None:
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--offline", "--no-deps", "--format-version", "1"], cwd=ROOT))
    names = ["bonsai-runtime", "bonsai-governor", "bonsai-platform"]
    packages = {item["name"]: item for item in metadata["packages"]}
    forbidden = re.compile(
        r"bonsai_reference|primitive_adapter|linear_adapter|linear_control|bonsai-linear|environment_adapter|BRDC[-_]?1",
        re.IGNORECASE,
    )
    audited = {}
    for name in names:
        package = packages[name]
        dependencies = sorted(dep["name"] for dep in package["dependencies"] if dep["kind"] != "dev")
        assert not any(forbidden.search(dep) for dep in dependencies)
        sources = {}
        for path in sorted((ROOT / "crates" / name / "src").rglob("*.rs")):
            raw = path.read_bytes()
            assert not forbidden.search(raw.decode()), f"reference coupling: {path}"
            sources[path.relative_to(ROOT).as_posix()] = hashlib.sha256(raw).hexdigest()
        audited[name] = {"direct_dependencies": dependencies, "sources": sources}
    runner_sources = {}
    runner_paths = [ROOT / "crates/bonsai-xtask/src/experiment.rs"]
    runner_paths += sorted((ROOT / "crates/bonsai-xtask/src/experiment").rglob("*.rs"))
    for path in runner_paths:
        raw = path.read_bytes()
        assert not forbidden.search(raw.decode()), f"learner-specific authority branch: {path}"
        runner_sources[path.relative_to(ROOT).as_posix()] = hashlib.sha256(raw).hexdigest()
    fixture = ROOT / "fixtures/adapter-compatibility/v1"
    pins = json.loads((fixture / "historical-source-set.json").read_text())
    previous = hashlib.sha256((fixture / "previous_adapter_pb2.py").read_bytes()).hexdigest()
    assert previous == pins["source_files"]["python/bonsai-reference/src/bonsai/adapter/v1/adapter_pb2.py"]
    result = {"schema": "bonsai.adapter-dependency-audit/v1", "authority": audited, "runner_sources": runner_sources,
              "historical_binding_sha256": previous,
              "boundary": ("generic authority source and direct dependencies; "
                           "scientific bundle verifier is separately scoped")}
    (ROOT / "evidence/verification/bx-10/dependency-audit.json").write_text(
        json.dumps(result, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"authority_packages": names, "reference_imports": 0, "historical_binding_sha256": previous}))
if __name__ == "__main__":
    main()
