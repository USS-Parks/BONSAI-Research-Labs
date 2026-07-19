# Run lifecycle and recovery v1

Status: BR-05 supervisor lifecycle authority for schema epoch 1.

The supervisor records `created`, `running`, `degraded`, `terminating`, `completed`, `failed`, and `recovered` in a synchronized append-only JSON-lines journal. The state graph is explicit: a normal run reaches `completed` only through `terminating`; an interrupted live state first becomes `failed` and then `recovered`. A completed or recovered run is terminal.

## Durable transition boundary

Each agent transition uses one observer-owned transaction ID and at most one event segment:

1. a create-new intent binds the ID, reserved segment sequence, and SHA-256 of the opaque transition bytes;
2. the existing BC-09 segment writer appends, flushes, and synchronizes those complete bytes before the crash boundary, then publishes the segment without clobbering;
3. a create-new receipt binds the transition ID to the validated segment checksum;
4. a create-new consumed marker settles the transition.

Only one unsettled intent is allowed. Duplicate IDs, changed bytes, sequence conflicts, terminal-state events, and termination with an unsettled intent fail closed. Recovery never calls the adapter, returns telemetry to it, or resumes learning.

## Crash recovery

Opening an interrupted run validates the lifecycle journal, recovers every canonical `.open` segment through the BC-09 copy-and-publish recovery path, validates the contiguous immutable segment set, and settles intents from durable evidence:

- an intent with a validated segment becomes `preserved` and receives any missing receipt/consumed marker;
- an intent without a segment becomes `abandoned_before_append` and is never executed;
- a corrupt journal, segment, or inconsistent receipt stops recovery with a stable failure code and preserves the source evidence.

After settlement, an interrupted active run records `failed` and `recovered` with bounded reason codes. This produces a valid runtime evidence bundle: a legal journal, a valid immutable event-segment bundle, and one durable outcome per prepared transition. It is not by itself a BC-12 claim-ready scientific result bundle; later assembly still supplies the experiment manifest, track declaration, inventory, resource policy, failures, and metric provenance required by whole-bundle validation.

## Verification boundary

`fixtures/run-lifecycle/v1/expected-outcomes.json` freezes kills after every lifecycle state and after intent, frame append, segment finalization, receipt, and consumed-marker boundaries. Tests reopen each directory, require valid recovered segments, require `failed`/`recovered` terminal evidence for interrupted runs, and prove the segment count cannot increase when the same transition is presented again.

BR-05 does not resume agent learning from observer telemetry, implement observer replay, enforce resource budgets, or claim an adversarial operating-system sandbox. BR-06 owns agent/observer launch and data isolation.
