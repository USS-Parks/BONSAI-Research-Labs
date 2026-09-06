"""Optional FrozenLake adapter using the shared BONSAI online environment protocol."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import cast

from bonsai_reference.adapter_protocol import serve
from bonsai_reference.environment_adapter import MAXIMUM, SessionAdapter
from bonsai_reference.gymnasium_session import GymnasiumSession, GymnasiumSpec
from bonsai_reference.scenario import ScenarioError


def load_spec(path: Path) -> tuple[GymnasiumSpec, bytes]:
    with path.open("rb") as stream:
        payload = stream.read(MAXIMUM + 1)
    if len(payload) > MAXIMUM:
        raise ScenarioError("SCENARIO_CONFIGURATION_TOO_LARGE")
    raw: object = json.loads(payload)
    if not isinstance(raw, dict):
        raise ScenarioError("GYMNASIUM_CONFIGURATION_INVALID")
    values = cast(dict[str, object], raw)
    if set(values) != set(GymnasiumSpec.__dataclass_fields__):
        raise ScenarioError("GYMNASIUM_CONFIGURATION_INVALID")
    spec = GymnasiumSpec(
        scenario_id=cast(str, values["scenario_id"]), version=cast(str, values["version"]),
        seed=cast(int, values["seed"]), horizon=cast(int, values["horizon"]),
        action_count=cast(int, values["action_count"]), observation_width=cast(int, values["observation_width"]),
        environment_id=cast(str, values["environment_id"]), gymnasium_version=cast(str, values["gymnasium_version"]),
        map_name=cast(str, values["map_name"]), is_slippery=cast(bool, values["is_slippery"]),
    )
    spec.validate()
    return spec, hashlib.sha256(payload).digest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", type=Path, required=True)
    arguments = parser.parse_args()
    spec, identity = load_spec(arguments.spec)
    return serve(SessionAdapter(GymnasiumSession(spec), identity))


if __name__ == "__main__":
    raise SystemExit(main())
