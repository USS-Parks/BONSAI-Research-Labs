# Incremental lineage validation

BX-07 replaces per-event history cloning and replay in
`ArtifactLifecycleRegistry::apply` with `IncrementalLineageValidator`.
Both paths use the existing contract checks and error codes. Full replay remains
available through `validate_artifact_lineage_trace` and registry reconstruction.

The persistent validator keeps current artifact state, revision ownership,
parent relations, consumer links, and used history identities. A new birth
cannot create a cycle because it has no prior identity and every parent revision
must already belong to an existing artifact. A revision checks only newly added
parent edges: an edge from child to parent creates a cycle exactly when that
parent can already reach the child. This traversal is iterative and visits only
the reachable ancestry. The replay oracle retains its independent whole-graph
cycle traversal.

All validation precedes semantic mutation. Rejected cost and utility measurements
do not reserve their history IDs; rejected revisions do not reserve revision IDs
or advance sequence numbers. The registry projection uses the contract-validated
preconditions and preserves its accepted event prefix on rejection.

The generated differential corpus uses 12 deterministic seeds and 240 candidate
events per seed. All 2,880 decisions agree with full replay: 1,810 accepted and
1,070 rejected. Accepted registry snapshots agree with full reconstruction.
Rejected persistent validator states, registry snapshots, and event-prefix
lengths remain unchanged. Separate cases cover retrying a rejected measurement
with the same ID and rejecting a cycle through a diamond-shaped graph.

## Scaling evidence

Each workload runs in a fresh process, serially, for 1,000, 10,000, and 100,000
prefix events. Three trials per size and workload on each OS yield 72 trials.
All operation-count and expected-result gates passed. Builds use the debug
profile; these are diagnostic measurements, not optimized production benchmarks.

| Probe after 100,000 events | Windows median | WSL2 Linux median | Validation work |
|---|---:|---:|---|
| Append one cost record | 22.5 µs | 38.7 µs | 2 artifact lookups, 1 history lookup, 0 ancestry nodes |
| Add an independent root | 26.9 µs | 48.8 µs | 1 artifact lookup, 1 revision lookup, 0 ancestry nodes |
| Reject a cycle through a chain | 259.8 ms | 228.9 ms | 100,000 ancestry nodes, 99,999 edges |
| Add an edge into branching ancestry | 276.4 ms | 341.8 ms | 99,999 ancestry nodes, 199,995 edges |

Every incremental admission reported one event, zero whole-graph scans, and zero
graph clones. Routine-probe lookup counts were unchanged across all three sizes.
This does not assert constant-time map operations, allocation, payload copying,
storage append, or arbitrary ancestry traversal. The measured timer covers the
registry admission; the work counters describe validation probes and traversed
relations.

RSS is measured with the existing portable process-tree collector before
construction, after construction, and after the probe. These are current RSS
snapshots, not peak RSS. At 100,000 cost-history events, median post-construction
RSS was 171,581,440 bytes on Windows and 158,740,480 bytes on WSL2 Linux. Retained
events and historical registry data still grow with history; bounded persistence
is BX-08. No physical-platform qualification or scientific claim is inferred.

[Scaling evidence archive](../../evidence/verification/bx-07/scaling-evidence.zip)
contains the 72 raw trials, both source/binary identities, and nine exact measured
source files. It is 104,993 bytes with SHA-256
`6a7b99ae8cc3bdbb2589a9fe932e8af7b0cd63841365a1fbb0fd349d971881ed`.
All 156 entries were re-read and hashed.

To reproduce one bounded workload:

```text
cargo run --offline -p bonsai-lineage --example incremental_scaling -- history 100000
```

For the complete native Windows corpus, build that example and run
`uv run --frozen python evidence/verification/bx-07/scaling.py`.
The Linux entry point is `sh evidence/verification/bx-07/scaling-linux.sh`.
Both retain their outputs in a timestamped directory under `target`.
