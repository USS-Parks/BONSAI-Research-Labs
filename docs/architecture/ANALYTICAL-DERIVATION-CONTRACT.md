# Arrow and Parquet analytical derivation contract v1

BC-11 defines four typed, non-authoritative analytical tables materialized as Parquet with Arrow schemas. Event segments, manifests, metric contracts, artifact lifecycle events, governor decisions, and content-addressed blobs remain authoritative. A derived table may be deleted and regenerated; it may never repair, replace, or reinterpret missing raw evidence silently.

## Frozen column semantics

All fields use Arrow `Utf8`, `UInt64`, or `Float64`. Required/optional status is part of the schema hash. Identifiers and hashes are lowercase text where the upstream contract requires it; unsigned sequences and nanosecond timestamps retain the full `UInt64` domain. Unavailable metric values are null and remain distinct from numeric zero. Non-finite metric values are rejected.

| Table | Required columns | Optional columns |
|---|---|---|
| `event` | `run_id`, `source_id`, `source_sequence`, `event_type`, `monotonic_time_ns`, `payload_sha256` | `wall_time_unix_ns` |
| `metric` | `run_id`, `metric_id`, `metric_version`, `unit`, `availability`, `input_sha256` | `value` |
| `lineage` | `artifact_id`, `revision`, `artifact_type`, `disposition` | `parent_artifact_id`, `consumer_artifact_id` |
| `decision` | `run_id`, `decision_id`, `policy_version`, `outcome`, `reason_code`, `observed_state_sha256`, `requested_work_sha256` | none |

Each schema has a stable textual contract identifier and SHA-256 digest. A field rename, reordering, type change, or nullability change therefore invalidates the stored schema identity rather than drifting silently.

## Required Parquet provenance

Every file carries namespaced key/value metadata for `bonsai.derivation/v1`, table kind, schema SHA-256, semantic SHA-256, a sorted nonempty set of authoritative input SHA-256 values, producer ID/version, and row count. Duplicate inputs or metadata keys fail. Materialization uses create-new semantics and never replaces an existing derivative.

Semantic identity hashes the frozen schema contract, total row count, null markers, type markers, exact UTF-8 bytes, unsigned integer bytes, and finite IEEE-754 value bits. It is independent of Parquet page boundaries and physical encoding. Validation reads every Arrow batch and recomputes this identity.

Current authoritative input hashes that differ from the stored set produce `DERIVATION_INPUT_MISMATCH`. A producer/version mismatch or recomputed row/semantic mismatch produces `DERIVATION_STALE`. Table-kind and schema mismatches have separate outcomes. These statuses prevent a stale or wrong-input table from being mistaken for current evidence.

The committed acceptance matrix is [`fixtures/derivations/v1/expected-outcomes.json`](../../fixtures/derivations/v1/expected-outcomes.json). Hosted Windows, Linux, macOS ARM64, and macOS Intel jobs materialize all four schemas, validate round trips, prove semantic equality across independent regeneration, and detect wrong inputs and stale producers.
