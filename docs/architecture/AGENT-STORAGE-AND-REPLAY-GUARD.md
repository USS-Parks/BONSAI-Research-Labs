# Agent storage and replay guard v1

Status: BQ-06 authority for metered agent persistence and transition-retention classification.

The external governor brokers agent-owned persistence under one fixed byte and file budget. The broker admits model parameters and bounded algorithm state, classifies transition-like retention, denies observer-tree paths, and fails closed on path traversal or symlink targets. It does not ban legitimate learned parameters, optimize scientific utility, or claim an adversarial operating-system sandbox.

## Policy invariants

A `StoragePolicy` must identify itself, declare positive `max_bytes`, `max_files`, and `max_bounded_state_bytes`, and keep replay capacity consistent with `allow_transition_replay`. Track A policies set replay capacity to zero. Replay-capable comparator policies must declare a positive capacity. Malformed identity or bounds fail before broker state exists.

## Classification and decisions

Each persistence request carries a logical path, byte/file deltas, declared retention kind, path shape, and content signals. The broker classifies retention before metering:

- transition replay from declared kind, transition-shaped content signals, or replay-buffer path names;
- model parameters when declared and free of transition signals;
- bounded algorithm state when declared, sized within the policy bound, and free of retained transitions;
- otherwise unclassified, which rejects without meter growth.

Observer paths, lexical traversal, and symlink shapes reject with stable codes and leave counters unchanged. Byte or file projections above the policy caps reject with `STORAGE_BYTE_BUDGET_EXHAUSTED` or `STORAGE_FILE_BUDGET_EXHAUSTED`. Under a zero-replay policy, classified transition retention rejects with `TRANSITION_REPLAY_RETENTION_DENIED` while retaining the detected capacity on the decision. Only admission commits checked meter growth.

## Track consequence

Admitted transition retention accumulates `classified_transition_capacity` and projects BC-05 runtime facts through `track_declaration_overlay`. Derived track becomes `B` when capacity is positive. Denied detection still records classification on the decision so silent replay cannot hide, but Track A meters and derived facts remain clean when retention is refused.

Live helpers `inspect_path_shape` and `resolve_under_work_root` support supervisor integration without following symlinks. The committed corpus at `fixtures/agent-storage/v1/expected-outcomes.json` freezes the adversarial admit/classify/deny sequence.

BQ-06 is an agent-persistence and retention-classification guard. Platform hard limits, hostile-native sandboxing, and claim-ladder verdicts remain outside this prompt.
