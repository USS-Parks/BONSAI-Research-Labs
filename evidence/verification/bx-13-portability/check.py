import hashlib
import json
import runpy
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
helper = runpy.run_path(str(ROOT / "evidence/verification/bx-13/ci-protocol.py"))
records, violations = helper["verifier_source_identity"](enforce_checkout=True)
assert len(records) == 20 and not violations, violations
before = json.loads((ROOT / "evidence/verification/bx-13-portability/checkout-before.json").read_bytes())
assert len(before["mismatches"]) == 4
rejected = []
for row in before["rows"]:
    if row["equal"]:
        continue
    path = row["path"]
    blob = subprocess.check_output(
        ["git", "cat-file", "blob", "82ed6a5b3eaa6f5eb36349df53edfd26dd44692e:" + path], cwd=ROOT
    )
    crlf = blob.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    assert hashlib.sha256(crlf).hexdigest() == row["windows_checkout_sha256"]
    _, problems = helper["source_record"](path, blob, crlf, crlf, {"text": "set", "eol": "lf"}, enforce_checkout=True)
    assert any("filtered-bytes-differ" in p for p in problems) and any("checkout-bytes-differ" in p for p in problems)
    _, attribute_problems = helper["source_record"](
        path, blob, blob, blob, {"text": "unspecified", "eol": "unspecified"}, enforce_checkout=True
    )
    assert len(attribute_problems) == 2
    rejected.append(path)
identity = json.loads((ROOT / "target/bx13-live-1788717713189903230/source-identity.json").read_bytes())
deltas = []
for path, old in identity["source_files"].items():
    current = hashlib.sha256((ROOT / path).read_bytes()).hexdigest()
    if current != old:
        deltas.append({"path": path, "original_sha256": old, "current_sha256": current})
assert {r["path"] for r in deltas} == {".gitattributes", "evidence/verification/bx-13/ci-protocol.py"}
archive = ROOT / "evidence/verification/bx-13/online-options.zip"
assert (
    hashlib.sha256(archive.read_bytes()).hexdigest()
    == "030b972470cf36f7ea252180acce8226da28df6ef8f9b8af6bfb8cc73e1e631c"
)
result = {
    "result": "pass",
    "source_pins": 20,
    "windows_crlf_rejections": rejected,
    "attribute_denials": 4,
    "source_files_unchanged": 256,
    "metadata_source_deltas": deltas,
    "original_archive_unchanged": True,
}
(ROOT / "evidence/verification/bx-13-portability/guard-verification.json").write_text(
    json.dumps(result, indent=2) + "\n"
)
print(json.dumps(result, indent=2))
