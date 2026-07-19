"""Validate local Markdown links and mandatory BONSAI STS warnings."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]
LINK = re.compile(r"(?<!!)\[[^]]*]\(([^)]+)\)")
EXTERNAL_SCHEMES = ("http://", "https://", "mailto:")
STS_MARKERS = ("run it STS", "run M0 STS")
EXCLUDED_DIRECTORIES = {".git", ".venv", "target"}


def local_link_errors(markdown: Path) -> list[str]:
    errors: list[str] = []
    text = markdown.read_text(encoding="utf-8")
    for raw_target in LINK.findall(text):
        target = raw_target.strip().strip("<>").split("#", maxsplit=1)[0]
        if not target or target.startswith(EXTERNAL_SCHEMES):
            continue
        resolved = (markdown.parent / unquote(target)).resolve()
        if not resolved.is_relative_to(ROOT):
            errors.append(f"{markdown.relative_to(ROOT)}: link escapes repository: {raw_target}")
        elif not resolved.exists():
            errors.append(f"{markdown.relative_to(ROOT)}: missing local link: {raw_target}")
    return errors


def main() -> int:
    errors: list[str] = []
    markdown_files = sorted(
        path
        for path in ROOT.rglob("*.md")
        if not EXCLUDED_DIRECTORIES.intersection(path.relative_to(ROOT).parts)
    )
    for markdown in markdown_files:
        errors.extend(local_link_errors(markdown))

    warning_files = (
        ROOT / "README.md",
        ROOT
        / "BONSAI Research Charter"
        / "BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md",
    )
    for warning_file in warning_files:
        text = warning_file.read_text(encoding="utf-8")
        if not any(marker in text for marker in STS_MARKERS):
            errors.append(f"{warning_file.relative_to(ROOT)}: missing explicit STS warning")

    if errors:
        print("documentation governance check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print(
        f"documentation governance check passed: {len(markdown_files)} Markdown files; "
        "local links resolve; STS warnings present"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
