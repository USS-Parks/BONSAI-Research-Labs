# BONSAI Protobuf evolution policy

Status: frozen for schema epoch 1 by BC-01.

This directory contains BONSAI's portable wire contracts. BC-01 defines how they evolve; BC-02 introduces the first epoch-1 message.

## Version model

- Every contract belongs to an integer **epoch** and a non-decreasing integer **minor** revision. Package and path names carry the epoch as `v<epoch>`; catalog metadata carries both values.
- A minor revision is additive only. It may add a new message, a new enum value, or a new optional field using a never-used field number. Application exhaustiveness still has to be reviewed.
- A breaking semantic or wire change requires a new epoch, a migration record, and retained support for reading the prior frozen fixture. Incrementing the epoch is not permission to destroy the original evidence.
- A schema change and its catalog/fixture update are one atomic commit. An undocumented schema change is invalid even if `protoc` accepts it.

## Field rules

- A field number is permanent once published. Renumbering is deletion plus creation and is rejected within an epoch.
- Deleted fields reserve both their number and name forever. A reserved number or name is never returned to service.
- Field type, presence, cardinality, oneof membership, and unit are schema semantics. They do not change in a minor revision.
- Numeric fields declare a unit. Dimensionless values use `1`; unavailable measurements use an explicit availability contract rather than a sentinel number.
- Fields 19000 through 19999 remain unavailable because Protobuf reserves them for its implementation.
- Consumers must tolerate unknown binary fields. A component that parses and reserializes an envelope must use a supported binary relay path proven to preserve them; ProtoJSON is not that path.

These constraints intentionally tighten the official Protobuf compatibility baseline. The upstream guidance identifies new fields as binary wire-safe, field renumbering as unsafe, and reservation of deleted numbers and names as the safe deletion practice. See the [proto3 updating guidance](https://protobuf.dev/programming-guides/proto3/#updating) and [Proto Best Practices](https://protobuf.dev/best-practices/dos-donts/).

## Compatibility gate

Run from the repository root:

```text
cargo xtask schema-check
```

The command compares the frozen epoch-1 baseline with one additive fixture and four prohibited-change fixtures. It succeeds only when the additive case is compatible and field renumbering, reserved-field reuse, silent unit change, and unversioned JSON are rejected with their named error codes. The fixture catalog is a checker input format, not a BONSAI domain contract.

Fixtures live in [`../fixtures/schema-compatibility/v1`](../fixtures/schema-compatibility/v1). The companion JSON and migration rules are in [`../schemas/README.md`](../schemas/README.md).

## Universal event envelope

[`bonsai/event/v1/envelope.proto`](bonsai/event/v1/envelope.proto) is the epoch-1 envelope authority. Identifiers are nonzero 16-byte UUID representations. `source_sequence` orders events only within one source; causal-parent IDs express a partial order. `monotonic_time_ns` is required and local to its clock domain. Optional Unix wall time is observational metadata and never establishes global order.

The envelope carries an event-type identifier, payload schema epoch/minor, raw payload, and SHA-256 payload digest. Availability uses the frozen measured/estimated/unavailable/excluded states. Precision declares the payload representation and, when meaningful, significant bits. Validation rejects malformed or zero IDs, a self-parent, zero monotonic time, nonpositive present wall time, invalid event type/version/availability/precision, and payload hash mismatch.

Rust bindings and the descriptor set are generated at build time from the same `.proto` using a vendored cross-platform `protoc`. Python conformance tests load that descriptor dynamically; no second hand-maintained binding exists.

The supported unknown-field relay is deliberately byte-preserving: it decodes and validates known fields, then forwards the original binary bytes. Prost decode/re-encode and ProtoJSON are not relay paths because they cannot prove retention of unknown fields. Tests append an unknown field 99 and require byte-for-byte survival through Python → Rust → Python.

## Governor decision event

[`bonsai/governor/v1/decision.proto`](bonsai/governor/v1/decision.proto) is the BC-06 wire authority for immutable external-governor decisions. Every event binds a nonzero decision/run identity to an exact resource-policy identity, version, and canonical SHA-256 hash. It records monotonic decision time, the complete requested work vector, the observed budget state used by the decision, measured, estimated, or unavailable basis, affected limit IDs, a stable reason code, and exactly one `admit`, `defer`, `throttle`, `reject`, or `terminate` outcome.

Rolling-window observations carry their start and duration; per-event observations require a triggering event identity. Measured and estimated observations require an explicitly present numeric value, estimated observations additionally require estimator identity/version, and unavailable observations prohibit a numeric value and require an availability reason. Deferred decisions require a future monotonic retry time, throttled decisions require nonzero allocations strictly below the corresponding request, and the other outcomes prohibit action parameters. `bonsai_contracts::resource::reconstruct_governor_decision` validates these invariants and returns the four evidence surfaces needed to reconstruct the decision input: exact policy reference, observed state, requested work, and reason code.

This is an evidence contract, not a governor implementation. BC-06 neither measures counters nor enforces a backend limit.

## Cognitive artifact and lineage events

[`bonsai/artifact/v1/lineage.proto`](bonsai/artifact/v1/lineage.proto) is the BC-07 payload authority for cognitive-artifact identity, immutable revisions, provenance parents, consumers, resource-cost history, utility history, and dispositions. It covers the seven epoch-1 artifact types: feature, subproblem, option, model, planner, policy, and value function. The payload is carried by the universal event envelope; it does not duplicate envelope run/event identity.

`artifact_id` remains stable across an artifact's lifecycle. A birth creates lifecycle sequence 1 and one globally unique `artifact_revision_id`; every revision creates a new immutable revision ID and names the immediately preceding revision. Other lifecycle records target the current revision. Parent references name an already observed artifact revision and form an acyclic provenance DAG. The revision chain is separate, so a revision never lists its own artifact as a provenance parent.

Every birth, revision, consumer link change, cost/utility observation, and disposition carries producer/version/source-event provenance. Cost and utility observations distinguish measured, estimated, unavailable, and excluded values: unavailable is never numeric zero, and estimates name their estimator. Formal metric-result semantics remain BC-08 scope.

Retained and deprioritized dispositions are nonterminal. Replaced, retired, and removed are terminal; a replaced artifact names an already born successor identity. After terminal disposition, the same stable identity cannot be revised or relinked. A newly born identity may cite the terminal artifact revision as provenance. Delayed immutable cost or utility observations may still be recorded against the terminal revision and do not resurrect it.

`bonsai_contracts::lineage::validate_artifact_lineage_trace` is a pure schema-conformance validator, not the runtime registry scheduled for BR-07. Exhaustive generated lifecycle tests reject orphan revisions, missing provenance, provenance cycles of lengths two through six, and resurrection after every terminal disposition while proving that a replacement under a new identity remains valid. The schema records learning artifacts without prescribing their representation or learning algorithm.
