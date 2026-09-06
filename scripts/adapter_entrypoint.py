"""Trusted source bootstrap for isolated Python adapter children (-I -B)."""

import runpy
import sys
from pathlib import Path

source = Path(__file__).resolve().parents[1] / "python" / "bonsai-reference" / "src"
sys.path.insert(0, str(source))
if len(sys.argv) < 2 or sys.argv[1] not in {
    "environment_adapter", "gymnasium_adapter", "primitive_adapter", "linear_adapter",
}:
    raise SystemExit("expected a registered adapter name")
module = "bonsai_reference." + sys.argv.pop(1)
runpy.run_module(module, run_name="__main__")
