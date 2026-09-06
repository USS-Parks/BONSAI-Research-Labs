"""Record optional dependency presence and pinned installed provenance."""
import hashlib
import importlib.metadata
import importlib.util
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[3]
if sys.argv[1:] == ["--core"]:
    sys.path.insert(0, str(root / "python/bonsai-reference/src"))
    assert importlib.util.find_spec("gymnasium") is None
    for module in ["environment_adapter", "primitive_adapter", "linear_adapter"]:
        importlib.import_module("bonsai_reference." + module)
    print(json.dumps({"core_imports": "pass", "gymnasium_installed": False}))
else:
    assert not sys.argv[1:]
    result = {}
    for name, version in {"gymnasium": "1.3.0", "numpy": "2.5.2",
                          "cloudpickle": "3.1.2", "farama-notifications": "0.0.6"}.items():
        dist = importlib.metadata.distribution(name)
        assert dist.version == version
        metadata = dist.read_text("METADATA")
        assert metadata is not None
        result[name] = {"version": version, "metadata_sha256": hashlib.sha256(metadata.encode()).hexdigest(),
                        "license": dist.metadata.get("License-Expression") or dist.metadata.get("License"),
                        "project_urls": dist.metadata.get_all("Project-URL")}
    print(json.dumps({"optional_packages": result,
                      "lock_sha256": hashlib.sha256((root / "uv.lock").read_bytes()).hexdigest()}, indent=2))
