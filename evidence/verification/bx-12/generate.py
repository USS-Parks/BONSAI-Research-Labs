"""Generate independent audit bindings with the locked compiler."""

import importlib
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "scripts"))
compiler = importlib.import_module("check_python_protocol").compiler
output = ROOT / "target/bx12-audit-proto"
output.mkdir(exist_ok=True)
subprocess.run([
    str(compiler()), "--proto_path=proto", f"--python_out={output}",
    "proto/bonsai/artifact/v1/lineage.proto", "proto/bonsai/event/v1/envelope.proto",
], cwd=ROOT, check=True, timeout=30)
