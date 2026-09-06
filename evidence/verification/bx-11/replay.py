"""Independently replay observed Gymnasium actions against the pinned upstream API."""
import importlib
import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path.cwd()
sys.path.insert(0, str(ROOT / "target/bx10-audit-proto"))
wire = importlib.import_module("bonsai.adapter.v1.adapter_pb2")
importlib.import_module("bonsai.event.v1.envelope_pb2")
spec = importlib.util.spec_from_file_location("reader", ROOT / "evidence/verification/bx-05/reconcile.py")
reader = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reader)
gym = importlib.import_module("gymnasium")
assert gym.__version__ == "1.3.0"
batch = ROOT / sys.argv[1]
summary = json.loads((batch / "summary.json").read_text())
results = []
for row in summary["runs"]:
    case = ROOT / row["case"]
    events = reader.read_events(case / "run/observer/telemetry/segment-00000000000000000000.bseg")
    decoded = [(event.event_type, json.loads(event.payload)) for event in events]
    actions = [p for k, p in decoded if k == "run.action"]
    rewards = [p for k, p in decoded if k == "run.reward"]
    env = gym.make("FrozenLake-v1", map_name="4x4", is_slippery=False,
                   max_episode_steps=row["horizon"], render_mode=None)
    episode = None
    done = False
    flags = {"terminated": 0, "truncated": 0, "both": 0}
    traces = []
    for action, reward in zip(actions, rewards, strict=True):
        if action["episode"] != episode:
            assert episode is None or done
            episode = action["episode"]
            observed, _ = env.reset(seed=row["seed"] + episode)
            done = False
        obs = wire.CausalObservation.FromString(bytes.fromhex(action["observation_hex"]))
        transition = wire.CausalTransition.FromString(bytes.fromhex(reward["transition_hex"]))
        assert list(obs.observation) == [observed]
        assert list(obs.allowed_actions) == [0, 1, 2, 3]
        observed, value, terminated, truncated, _ = env.step(action["action"])
        done = terminated or truncated
        assert transition.action == action["action"]
        assert transition.reward == value == reward["reward"]
        assert transition.terminated == terminated and transition.truncated == truncated
        assert list(transition.next.observation) == [observed]
        assert list(transition.next.allowed_actions) == ([] if done else [0, 1, 2, 3])
        flags["terminated"] += bool(terminated)
        flags["truncated"] += bool(truncated)
        flags["both"] += bool(terminated and truncated)
        traces.append({"episode": episode, "step": action["step"], "action": action["action"],
                       "observation": observed, "reward": value,
                       "terminated": bool(terminated), "truncated": bool(truncated)})
    env.close()
    assert len(traces) == 100
    results.append({"case": row["case"], "steps": len(traces), "endings": flags, "traces": traces})
output = {"schema": "bonsai.gymnasium-independent-replay/v1", "runs": results,
          "scientific_quality_claim": False}
with (batch / "environment-replay.json").open("x") as stream:
    json.dump(output, stream, indent=2)
print(json.dumps({"runs": len(results), "steps": sum(row["steps"] for row in results)}))
