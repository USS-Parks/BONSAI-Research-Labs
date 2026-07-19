# BONSAI architecture decision records

ADRs are accepted design constraints, not evidence that the design has been implemented. Each settled PSPR decision D-01 through D-21 maps to exactly one ADR below.

| ADR | Status | Decision IDs | Subject |
|---|---|---|---|
| [0001](0001-repository-and-publication-boundary.md) | accepted | D-01, D-09 | Repository identity, license, visibility, publication |
| [0002](0002-runtime-and-process-boundary.md) | accepted | D-02, D-03, D-21 | Rust/Python stack, adapter process boundary, containment scope |
| [0003](0003-storage-schema-and-reporting.md) | accepted | D-04, D-05, D-06 | Evidence storage, evolution, reports |
| [0004](0004-packaging-and-offline-reproducibility.md) | accepted | D-07, D-08 | Distribution and offline restoration |
| [0005](0005-platform-and-measurement-policy.md) | accepted | D-10, D-11, D-12, D-13, D-14 | Host matrix, counters, overhead, energy |
| [0006](0006-statistics-and-resource-profiles.md) | accepted | D-15, D-16 | Statistical requirements and run profiles |
| [0007](0007-reference-science-and-attribution.md) | accepted | D-17, D-18, D-19, D-20 | BRDC-1, scenarios, consequential planning, utility |

## Decision lifecycle

All records are owned by BONSAI maintainers and accepted from PSPR v0.1 on 2026-07-18. An ADR can be superseded only by a dated, user-approved PSPR addendum plus a new ADR that names the old record, affected prompts and evidence, and migration or invalidation consequences. Git history is retained; accepted text is not silently rewritten.
