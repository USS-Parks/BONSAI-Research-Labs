"""Exercise receipt and semantic rejection using copies of a real accepted run.

Repinned cases are controlled negative fixtures, not new operator attestations.
They deliberately update container hashes so the verifier must reject semantics.
"""
import hashlib
import importlib
import json
import shutil
import struct
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path[:0] = [str(ROOT / "target/bx06-audit-proto"), str(ROOT / "python/bonsai-reference/src")]
event_wire = importlib.import_module("bonsai.event.v1.envelope_pb2")
adapter_wire = importlib.import_module("bonsai.adapter.v1.adapter_pb2")
BINARY = ROOT / ("target/debug/bonsai-xtask.exe" if sys.platform == "win32"
                 else "target/x86_64-unknown-linux-gnu/debug/bonsai-xtask")
SEGMENT = "telemetry/segment-00000000000000000000.bseg"


def digest(data):
    return hashlib.sha256(data).digest()


def read(path):
    return json.loads(path.read_bytes())


def write(path, value):
    path.write_bytes(json.dumps(value, sort_keys=True, indent=2).encode())


def events(root):
    data = (root / SEGMENT).read_bytes()
    values = []
    offset = 60
    while offset < len(data) - 88:
        length = struct.unpack_from("<I", data, offset + 8)[0]
        values.append(event_wire.EventEnvelope.FromString(data[offset + 12:offset + 12 + length]))
        offset += 44 + length
    return data[:60], values


def replace_events(root, header, values):
    content = bytearray(header)
    previous = None
    for sequence, event in enumerate(values):
        event.source_sequence = sequence
        event.event_id = digest(event.run_id + struct.pack("<Q", sequence))[:16]
        del event.causal_parent_event_ids[:]
        if previous is not None:
            event.causal_parent_event_ids.append(previous)
        previous = event.event_id
        event.payload_sha256 = digest(event.payload)
        encoded = event.SerializeToString(deterministic=True)
        content.extend(b"BNSFRM01" + struct.pack("<I", len(encoded)) + encoded + digest(encoded))
    footer = b"BNSEND01" + struct.pack("<QQ", 0, len(values)) + digest(content)
    (root / SEGMENT).write_bytes(content + footer + digest(footer))


def mutate_event(root, predicate, mutate):
    header, values = events(root)
    selected = next(event for event in values if predicate(event.event_type, json.loads(event.payload)))
    payload = json.loads(selected.payload)
    mutate(payload)
    selected.payload = json.dumps(payload, separators=(",", ":"), sort_keys=True).encode()
    replace_events(root, header, values)


def seal(root):
    bundle = read(root / "bundle-manifest.json")
    for entry in bundle["files"]:
        entry["sha256"] = digest((root / entry["path"]).read_bytes()).hex()
    write(root / "bundle-manifest.json", bundle)
    index = read(root / "artifact-index.json")
    for entry in index["files"]:
        data = (root / entry["path"]).read_bytes()
        entry.update(bytes=len(data), sha256=digest(data).hex())
    write(root / "artifact-index.json", index)
    status = read(root / "run-status.json")
    status["artifact_index_sha256"] = digest((root / "artifact-index.json").read_bytes()).hex()
    write(root / "run-status.json", status)
    receipt = read(root / "run-receipt.json")
    receipt["index_sha256"] = status["artifact_index_sha256"]
    receipt["status_sha256"] = digest((root / "run-status.json").read_bytes()).hex()
    write(root / "run-receipt.json", receipt)
    return digest((root / "run-receipt.json").read_bytes()).hex()


def change_json(root, name, mutate):
    value = read(root / name)
    mutate(value)
    write(root / name, value)


def change_case(root, case):
    if case == "missing_controller":
        mutate_event(root, lambda kind, value: value.get("phase") == "controls_before_launch",
                     lambda value: value["agent"].pop("cpu_max"))
    elif case == "history_exposure":
        mutate_event(root, lambda kind, value: value.get("phase") == "agent_launch_policy",
                     lambda value: value["audit"]["arguments"].extend(["--log-forward", "/observer/history"]))
    elif case == "policy_substitution":
        change_json(root, "resource-policy.json", lambda value: value["limits"][0].update(soft_limit=4, hard_limit=4))
        changed = digest((root / "resource-policy.json").read_bytes()).hex()
        mutate_event(root, lambda kind, value: value.get("phase") == "controls_before_launch",
                     lambda value: value.update(policy_sha256=changed))
    elif case == "event_gap":
        header, values = events(root)
        removed = next(index for index, event in enumerate(values) if event.event_type == "run.reward")
        values.pop(removed)
        replace_events(root, header, values)
    elif case == "cpu_regression":
        mutate_event(root, lambda kind, value: kind == "run.resource" and value.get("phase") == "step",
                     lambda value: value.update(cpu_usage_before_usec=value["usage"]["cpu_usage_usec"] + 1))
    elif case == "replay_accounting":
        def change(value):
            frame = adapter_wire.AdapterFrame.FromString(bytes.fromhex(value["frame_hex"]))
            measured = adapter_wire.PrimitiveAccounting.FromString(frame.work_result.result)
            measured.replay_items_retained = 1
            frame.work_result.result = measured.SerializeToString(deterministic=True)
            frame.work_result.result_sha256 = digest(frame.work_result.result)
            value["frame_hex"] = frame.SerializeToString(deterministic=True).hex()

        mutate_event(root, lambda kind, value: kind == "run.protocol"
                     and value.get("phase") == "response_received"
                     and adapter_wire.AdapterFrame.FromString(
                         bytes.fromhex(value["frame_hex"])).WhichOneof("message") == "work_result",
                     change)
    elif case == "unknown_track":
        change_json(root, "experiment-manifest.json", lambda value: value["track"].update(declared_track="UNKNOWN"))
    elif case == "unknown_reference":
        change_json(root, "source-identity.json", lambda value: value["source_files"].update(
            {"python/bonsai-reference/src/bonsai_reference/primitive_adapter.py": "0" * 64}))
    elif case == "metric_substitution":
        change_json(root, "metric-estimate.json", lambda value: value["result"].update(value=999999))
    elif case == "incomplete_status":
        change_json(root, "run-status.json", lambda value: value.update(status="INCOMPLETE"))
    else:
        raise AssertionError(case)


def run_corpus(source):
    batch = ROOT / "target" / ("bx06-negative-" + str(time.time_ns()))
    batch.mkdir()
    trusted = digest((source / "run-receipt.json").read_bytes()).hex()
    # The source pin is supplied by the separately retained operator output.
    operator = read(source.parent.parent / "operator.stdout.txt")
    assert operator["receipt_sha256"] == trusted
    cases = {
        "positive": None,
        "modified_file": "RUN_FILE_HASH_MISMATCH",
        "modified_receipt": "RUN_RECEIPT_HASH_MISMATCH",
        "missing_controller": "RUN_CONTROLLER_MISSING_OR_CHANGED",
        "history_exposure": "RUN_INFORMATION_BOUNDARY_VIOLATION",
        "policy_substitution": "RUN_POLICY_LIMIT_SUBSTITUTED",
        "event_gap": "RUN_EVENT_ORDER_INVALID",
        "cpu_regression": "RUN_CPU_COUNTER_INVALID",
        "replay_accounting": "RUN_ACCOUNTING_INCONSISTENT",
        "unknown_track": "RUN_SCHEMA_INVALID",
        "unknown_reference": "RUN_REFERENCE_SOURCE_UNSUPPORTED",
        "metric_substitution": "RUN_METRIC_RECONSTRUCTION_MISMATCH",
        "incomplete_status": "RUN_NOT_COMPLETE",
    }
    results = []
    for case, expected in cases.items():
        destination = batch / case
        shutil.copytree(source, destination)
        pin = trusted
        if case == "modified_file":
            (destination / "agent-configuration.json").write_bytes(b"{}")
        elif case == "modified_receipt":
            (destination / "run-receipt.json").write_bytes(b"{}")
        elif case != "positive":
            change_case(destination, case)
            pin = seal(destination)
        command = [str(BINARY), "verify-run", "--root", str(destination), "--receipt-sha256", pin]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=60, check=False)
        (batch / (case + ".stdout.txt")).write_bytes(result.stdout)
        (batch / (case + ".stderr.txt")).write_bytes(result.stderr)
        if expected is None:
            verdict = json.loads(result.stdout) if result.stdout else {}
            passed = result.returncode == 0 and verdict.get("facts", {}).get("derived_track") == "A"
            if passed:
                passed = all(row["c0"] == row["c1"] == "pass" for row in verdict["claims"]["rows"])
        else:
            passed = result.returncode != 0 and expected.encode() in result.stderr and not result.stdout
        record = {"case": case, "expected": expected, "exit_code": result.returncode,
                  "receipt_sha256": pin,
                  "repinned_fixture": case not in {"positive", "modified_file", "modified_receipt"},
                  "pass": passed}
        results.append(record)
        print(json.dumps(record), flush=True)
    write(batch / "summary.json", results)
    print("retained corpus: " + str(batch), flush=True)
    return 0 if all(record["pass"] for record in results) else 1


if __name__ == "__main__":
    raise SystemExit(run_corpus(Path(sys.argv[1])))
