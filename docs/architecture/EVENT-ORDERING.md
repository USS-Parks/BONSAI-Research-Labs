# Event partial-order semantics v1

Status: BR-04 ordering authority for schema epoch 1.

BONSAI derives event order only from per-source sequences and explicit causal-parent identities. Unix wall time is observational metadata and never creates an edge. Cross-source events remain concurrent unless a causal path relates them.

## Inputs and deterministic collection

Each `ObservedEvent` contains its validated BC-02 envelope and the immutable arrival index assigned by ingestion. The caller may supply the collection in any iteration order; the engine first keys it by arrival index, then produces canonical identity-sorted outputs. Duplicate arrival indices are rejected because they would make the recorded arrival relation ambiguous.

This distinction permits randomized collection-order tests without erasing the arrival-relative meaning of `late`: the vector can be shuffled while each event keeps the arrival index recorded when it entered the observer.

## Edges and classes

A source-sequence edge joins two unique events from the same source only when their sequence values are contiguous. A causal-parent edge joins a present parent identity to its child. If both rules yield the same edge, the report retains both edge kinds.

The report explicitly classifies:

- `duplicate`: an event identity was observed more than once;
- `late`: an occurrence arrived after a higher sequence from the same source;
- `missing_parent`: a declared causal parent is absent from the observed set;
- `concurrent`: neither event reaches the other through source or causal edges;
- `clock_regression`: monotonic time decreases across a unique contiguous source sequence;
- `sequence_conflict`: distinct event identities claim the same source sequence;
- `sequence_gap`: adjacent observed source sequences are not contiguous;
- `cycle`: a source/causal path returns to an event identity.

Conflicted source-sequence groups do not receive invented source edges. Gaps likewise remain gaps. Missing parents do not become synthetic nodes. Cycles are reported rather than silently broken. These choices preserve ambiguity for later claim logic.

## Reachability and bounds

Concurrency is derived from graph reachability, not shared ancestors, timestamp proximity, arrival proximity, or lexicographic identity. All reports use sorted IDs, pairs, edges, and classes. The engine rejects a zero/exceeded observation bound or causal-parent fan-in bound before graph construction.

The committed matrix is `fixtures/event-ordering/v1/expected-outcomes.json`. Fixtures freeze duplicate, late, missing-parent, concurrency, clock-regression, conflict, gap, and cycle outcomes; reverse/rotate the input collection while preserving recorded arrival indices and require an exactly equal report across operating systems.
