"""Online primitive tabular learner using the shared online adapter."""
from __future__ import annotations

from bonsai_reference.control import PrimitiveTabularControl
from bonsai_reference.online_adapter import ACCOUNTING as ACCOUNTING
from bonsai_reference.online_adapter import OBSERVATION as OBSERVATION
from bonsai_reference.online_adapter import REWARD as REWARD
from bonsai_reference.online_adapter import OnlineAdapter, run_adapter
from bonsai_reference.online_adapter import capabilities as capabilities
from bonsai_reference.scenario import ScenarioError


class PrimitiveAdapter(OnlineAdapter):
    def __init__(self, action_count: int, configuration_sha256: bytes) -> None:
        super().__init__(PrimitiveTabularControl(action_count), action_count, configuration_sha256)


def create(value: dict[str, object], digest: bytes) -> PrimitiveAdapter:
    actions = value.get("action_count")
    if set(value) != {"action_count"} or type(actions) is not int or not 2 <= actions <= 256:
        raise ScenarioError("AGENT_CONFIGURATION_INVALID")
    return PrimitiveAdapter(actions, digest)


def main() -> int:
    return run_adapter(create)


if __name__ == "__main__":
    raise SystemExit(main())
