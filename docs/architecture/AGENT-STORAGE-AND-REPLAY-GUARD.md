# Agent storage and replay guard v1

Status: BQ-06 authority for metered agent persistence and transition-like retention classification.

The supervisor meters authorized writes into `agent/work`. Model parameters and bounded algorithm state remain legal persistence. Transition-shaped records are classified as replay buffers. Observer-tree targets, path traversal, and symlink ancestors or destinations fail closed. The broker does not ban learned parameters or ordinary algorithm counters.

## Policy and metering

A `StoragePolicy` names a stable identity and positive byte, file, and per-file bounds. The per-file bound cannot exceed the lifetime byte bound. Track A policies set `allow_replay=false`. Missing identity, zero bounds, or an inverted per-file bound fail before broker state exists.

Every persist request records declared class, inspected class, outcome, reason, and the exact byte/file meters before and after. Rejected writes leave both meters unchanged. Replacement of an existing regular file charges the size delta and does not consume another file slot.

## Classification

Payload inspection is authoritative for replay. A JSON document whose `kind` is `replay_buffer`, or that contains a `transitions` array of `{state, action, reward, next_state}` records, is a replay buffer even when the caller declared model parameters. Declared replay is also classified as replay. Model-parameter and algorithm-state documents remain allowed when they do not contain transition records.

When `allow_replay` is false, a classified replay buffer is rejected with `REPLAY_BUFFER_DETECTED` and is not written. Inspection of the work tree therefore cannot discover hidden transition retention that the broker admitted.

## Path and observer denial

Relative paths must use a bounded number of safe components. Absolute paths, `.`, `..`, empty components, and unsafe characters fail with `STORAGE_PATH_TRAVERSAL` or `STORAGE_REQUEST_INVALID`. Any symlink ancestor or destination fails with `STORAGE_SYMLINK_DENIED`. A resolved path under the observer tree fails with `STORAGE_OBSERVER_PATH_DENIED`.

The committed corpus at `fixtures/storage-guard/v1/expected-outcomes.json` freezes admitted parameter/state writes, replay and hidden-replay classification, per-file and file-count exhaustion, and the resulting work-tree inspection.

BQ-06 meters and classifies persistence at the BONSAI work-tree seam. It does not claim an adversarial OS sandbox. Native code may still use ambient filesystem APIs unless a later platform broker prevents it.
