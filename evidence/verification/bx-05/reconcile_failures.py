"""Check kernel and telemetry facts behind live failure classifications."""

import json
import sys
from pathlib import Path

from reconcile import read_events

batch = Path(sys.argv[1])
cases = sys.argv[2:] or ["exit", "memory", "storage", "delay", "cancel", "cancel_pending",
                        "wall", "observer", "finalization"]
results = []
for case in cases:
    observer = batch / case / "observer"
    status = json.loads((observer / "run-status.json").read_text())
    assert status["status"] == "INCOMPLETE"
    events = [(event.event_type, json.loads(event.payload))
              for event in read_events(observer / "telemetry/segment-00000000000000000000.bseg")]
    cleanup = [value for kind, value in events if kind == "run.resource" and value.get("phase") == "cleanup"]
    assert {value["adapter"] for value in cleanup} == {"agent", "environment"}
    assert all(not value["usage"]["populated"] and not value["usage"]["member_pids"] for value in cleanup)
    facts = {}
    if case == "memory":
        usage = next(value["agent"] for kind, value in events
                     if kind == "run.resource" and value.get("phase") == "before_cleanup")
        assert usage["memory_oom_kills"] > 0 and usage["memory_max_events"] > 0
        facts = {"oom_kills": usage["memory_oom_kills"], "memory_max_events": usage["memory_max_events"]}
    elif case == "storage":
        sample = next(value for kind, value in events
                      if kind == "run.resource" and value.get("agent_storage_bytes", 0) > 65_536)
        facts = {"agent_storage_bytes": sample["agent_storage_bytes"]}
    elif case == "delay":
        failure = next(value for kind, value in events
                       if kind == "run.protocol" and value.get("phase") == "exchange_failed")
        assert failure["reason_code"] == "agent:TRANSPORT_READ_TIMEOUT"
        assert failure["latency_ns"] >= 50_000_000
        facts = {"latency_ns": failure["latency_ns"]}
    elif case == "cancel":
        assert any(kind == "run.action" for kind, _ in events)
        facts = {"actual_action_before_cancel": True}
    elif case == "cancel_pending":
        assert (batch / case / "agent/work/pending-initialization").exists()
        assert not any(kind == "run.action" for kind, _ in events)
        facts = {"initialization_entered_before_cancel": True}
    elif case == "wall":
        assert not any(kind == "run.protocol" for kind, _ in events)
        facts = {"expired_before_launch": True}
    elif case == "observer":
        count = sum(path.stat().st_size for path in observer.rglob("*") if path.is_file())
        assert count <= 262_144
        facts = {"observer_bytes": count}
    elif case == "finalization":
        assert status["reason_code"] == "RUN_FINALIZATION_FAILED"
        assert (observer / "reports/index.html").is_dir()
        facts = {"report_write_failure_retained": True}
    else:
        reaped = next(value for kind, value in events
                      if kind == "run.status" and value.get("phase") == "child_reaped" and value["adapter"] == "agent")
        assert reaped["exit_code"] == 23
        facts = {"agent_exit_code": reaped["exit_code"]}
    results.append({"case": case, "verdict": "FAILURE_FACTS_RECONCILED", **facts})
print(json.dumps(results, indent=2))
