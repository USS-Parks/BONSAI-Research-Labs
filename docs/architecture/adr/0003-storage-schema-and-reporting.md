# ADR 0003: Storage, schema evolution, and reporting

- Status: accepted
- Date: 2026-07-18
- Owner: BONSAI maintainers
- Source: approved PSPR v0.1

## D-04 — Combined evidence storage

Decision: canonical JSON manifests, append-only framed Protobuf event segments, content-addressed blobs, portable SQLite metadata/indexes, and derived Arrow/Parquet analytical tables form the storage model.

Rejected alternatives: one mutable database weakens append-only audit semantics; files-only storage makes indexing, recovery, and portable inspection harder.

Consequences: event segments remain authoritative; indexes and derived tables must be rebuildable; hashes and producer/input provenance bind every derivative.

## D-05 — Explicit schema evolution

Decision: schemas use explicit epochs, reserved Protobuf field numbers, additive minor evolution, migration fixtures, and JSON Schema 2020-12. Breaking change requires a major epoch and migrator.

Rejected alternatives: implicit migrations, field-number reuse, and unversioned JSON were rejected because they make old evidence ambiguous.

Consequences: compatibility checks precede schema changes; old fixtures stay readable or migrate explicitly; unit and meaning changes cannot masquerade as additive evolution.

## D-06 — Static canonical reports and optional viewer

Decision: static, self-contained HTML is the publication artifact. A local read-only dashboard/bundle viewer is also shipped but is never required for canonical runs.

Rejected alternatives: dashboard-only harms archival reproducibility; static-only slows interactive analysis.

Consequences: static and viewer values require parity; the viewer is read-only with no external egress; bundles remain inspectable offline.

## Supersession

Only a dated, user-approved PSPR addendum and replacement ADR may change canonical formats, evolution guarantees, or report authority.
