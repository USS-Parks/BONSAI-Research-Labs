"""Record release-candidate hashes. Does not tag, upload, or publish."""

from __future__ import annotations

import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evidence" / "release-candidate"
HASHED = (
    "Cargo.lock",
    "uv.lock",
    "docs/security/THREAT-MODEL.md",
    "docs/governance/DEPENDENCY-POLICY.md",
    "docs/releases/RC-0.0.0-NOTES.md",
    "docs/verification/M4-COMPLETION-AUDIT.md",
    "fixtures/threat-model/v1/boundaries.json",
    "fixtures/l-profile/v1/manifest.json",
    "fixtures/l-profile/v1/resource-policy.json",
    "fixtures/supply-chain/v1/waivers.json",
    "fixtures/host-attestation/v1/windows-not-run.json",
    "fixtures/host-attestation/v1/macos-not-run.json",
    "fixtures/host-attestation/v1/linux-not-run.json",
    "evidence/release-candidate/sbom-cargo.json",
    "evidence/release-candidate/sbom-python.json",
    "evidence/release-candidate/OD-03-NO-UPLOAD.txt",
    "evidence/release-candidate/SOURCE-PACKAGE.list",
)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    source_entries = [
        "Cargo.toml",
        "Cargo.lock",
        "pyproject.toml",
        "uv.lock",
        "python/bonsai-reference/src",
        "crates",
        "schemas",
        "docs",
        "scripts",
    ]
    (OUT / "SOURCE-PACKAGE.list").write_text("\n".join(source_entries) + "\n", encoding="utf-8")
    lines: list[str] = []
    for relative in HASHED:
        path = ROOT / relative
        if not path.is_file():
            print(f"package_rc: missing {relative}")
            return 1
        lines.append(f"{digest(path)}  {relative}")
    (OUT / "SHA256SUMS").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"package_rc: wrote {len(lines)} hashes; no tag, no registry upload")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
