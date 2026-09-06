"""Fixed-size linear online learner through the common BONSAI adapter."""
from __future__ import annotations

from bonsai_reference.linear_control import LinearControl
from bonsai_reference.online_adapter import OnlineAdapter, run_adapter
from bonsai_reference.scenario import ScenarioError


class LinearAdapter(OnlineAdapter):
    def __init__(self, actions: int, width: int, digest: bytes) -> None:
        super().__init__(LinearControl(actions, width), actions, digest)


def create(value: dict[str, object], digest: bytes) -> LinearAdapter:
    actions = value.get("action_count")
    width = value.get("observation_width")
    if (set(value) != {"action_count", "observation_width"}
            or type(actions) is not int or not 2 <= actions <= 256
            or type(width) is not int or not 1 <= width <= 256):
        raise ScenarioError("AGENT_CONFIGURATION_INVALID")
    return LinearAdapter(actions, width, digest)


def main() -> int:
    return run_adapter(create)


if __name__ == "__main__":
    raise SystemExit(main())
