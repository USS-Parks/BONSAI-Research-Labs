"""Bounded live BX-05 fault fixture; never used by normal operator manifests."""

import importlib
import os
import sys
import time
from pathlib import Path

repository = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(repository / "python/bonsai-reference/src"))
primitive_adapter = importlib.import_module("bonsai_reference.primitive_adapter")

fault = sys.argv.pop(1)
work = Path(sys.argv[sys.argv.index("--bonsai-work-dir") + 1])


class FaultAdapter(primitive_adapter.PrimitiveAdapter):
    injected = False

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        if fault == "cancel_pending":
            (work / "pending-initialization").touch()
            time.sleep(60)
        if fault == "memory":
            self.retained_fault = bytearray(256 * 1024 * 1024)

    def _step(self, request):
        if not self.injected:
            self.injected = True
            if fault == "exit":
                os._exit(23)
            elif fault == "storage":
                (work / "quota-overrun.bin").write_bytes(b"x" * 65_537)
            elif fault == "delay":
                time.sleep(0.075)
            else:
                raise RuntimeError("unknown fixture fault")
        return super()._step(request)


primitive_adapter.PrimitiveAdapter = FaultAdapter
raise SystemExit(primitive_adapter.main())
