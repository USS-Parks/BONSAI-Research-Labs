# Event ingestion validation v1

Status: BR-03 authority for validation before immutable append.

`bonsai-ingest` is the only supported route from an encoded adapter event into an open `bonsai-bundle` segment. The ingestor borrows the segment writer, validates the complete candidate, and calls `append` only after every applicable contract, authorization, lifecycle, size, and rate check passes. It never repairs or rewrites submitted evidence.

## Validation order

The validator applies these fail-closed checks:

1. The run lifecycle is `running`.
2. Encoded envelope bytes do not exceed the policy bound.
3. The BC-02 event envelope decodes and its IDs, times, event type, schema version, availability, precision, and payload SHA-256 are valid.
4. The event run identity equals the active run.
5. The source identity is manifest-authorized.
6. The source is authorized for the declared event type and payload size.
7. Causal-parent count is bounded.
8. Event type, schema epoch, and minor revision are registered and authorized.
9. Observer arrival time is non-regressing and the source remains within its fixed-window event-rate bound.
10. The original encoded bytes fit and append to the immutable segment.

The rate counter advances only after append succeeds. Event source sequence, duplicate identity, causal completeness, concurrency, and event-clock regression remain BR-04 ordering facts; BR-03 does not fabricate a total order or discard evidence needed to classify them.

## Lifecycle and rejection evidence

The ingestion lifecycle is `created → running → terminating → stopped`. Only `running` accepts events. Repeated or reversed transitions fail. Terminating and stopped runs produce `INGEST_LIFECYCLE_PRECONDITION` without touching the segment.

Every rejection returns a stable code plus encoded byte count and, when decoding was safe, fixed-size source and event identities. The observer ledger stores deterministic JSON records under both a record-count and total-byte cap. It evicts the oldest retained rejection when necessary and increments a saturating dropped-record count, preventing malformed-input floods from amplifying observer storage. It retains no arbitrary adapter-provided detail.

## Verification and boundaries

The committed matrix is `fixtures/event-ingest/v1/expected-outcomes.json`. Conformance proves exact run/source/type/schema/hash/rate/lifecycle outcomes, confirms that a mixed invalid corpus appends zero invalid frames, and feeds 2,048 deterministic pseudo-random byte strings through `catch_unwind` without a panic or append.

BR-03 does not interpret payload-domain semantics beyond registered schema identity, repair invalid events, classify partial order, recover a run, or grant filesystem/process access. BR-04 owns ordering; BR-05 owns lifecycle recovery; BR-06 owns agent/observer data isolation.
