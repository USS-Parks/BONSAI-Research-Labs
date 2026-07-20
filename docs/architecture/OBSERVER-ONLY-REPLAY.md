# Observer-only replay and analysis classification v1

Status: BR-09 authority for deterministic observer replay in schema epoch 1.

BONSAI may replay immutable telemetry to regenerate analytical tables and reports. That replay is an observer capability, never an agent learning capability. The runtime owns both the analysis path and the denial path so a Track A declaration cannot hide observer-derived input.

## Accepted input and canonical order

`ObserverReplayAnalyzer` accepts validated `EventEnvelope` values from exactly one run and version-pinned metric inputs. It reuses the BR-04 partial-order classifier, rejects duplicate events, missing causal parents, per-source sequence conflicts or gaps, and cycles, then performs a stable topological sort with event ID as the tie-break for concurrent events. Arrival order and wall-clock order do not choose the analytical row order.

The canonical event stream is length-framed and hashed with SHA-256 under a versioned replay-source domain. That source hash is stored as derivation provenance for both the event and metric Parquet tables. Metrics are computed only through the BK-01 registry. The BV-02 report generator renders those metric results; replay contains no independent report calculation path.

## Observer ownership and artifact sealing

Derived Parquet files are created beneath `observer/index`; machine JSON and self-contained HTML are created beneath `observer/reports`. Fixed filenames and a digest-derived directory name prevent path injection. Every file uses create-new semantics, so an existing replay is not replaced.

Public replay results contain only run/source identities, derivation summaries, byte counts, content hashes, and versioned seals. They contain no artifact bytes or filesystem paths. Each seal binds the artifact kind, run ID, canonical source hash, and content hash under `bonsai.observer-replay-seal/v1`.

The analyzer never writes beneath `agent/inputs` or `agent/work`. A matching observer-index or observer-report route is allowed. Any proposed route to agent input or protocol feedback calls the BR-06 denial seam, returns `OBSERVER_ACCESS_DENIED`, records the observer-data-access fact, and derives `INDETERMINATE_TRACK` with `OBSERVER_DATA_BOUNDARY_VIOLATION`.

## Verification and boundary

Fixtures prove that arrival permutations reproduce identical table summaries, file hashes, report hashes, and seals; emitted Parquet files validate against exact source provenance; every agent route is denied and becomes indeterminate; agent roots remain unchanged; malformed or incomplete orders fail closed; and repeated output cannot clobber prior evidence.

Observer replay is not agent training replay, an offline convergence pass, or evidence that an abstraction is useful. BR-09 establishes deterministic analysis and the one-way data boundary only. BQ-06 separately governs agent persistence and transition-like retention, and later claim rules decide scientific eligibility.
