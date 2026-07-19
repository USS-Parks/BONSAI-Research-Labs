# ADR 0004: Packaging and offline reproducibility

- Status: accepted
- Date: 2026-07-18
- Owner: BONSAI maintainers
- Source: approved PSPR v0.1

## D-07 — Distribution forms

Decision: ship native Rust CLI binaries for required hosts, Python wheels and source distributions for adapters, and the complete source repository. OCI images are conveniences, never core-run requirements.

Rejected alternatives: binary-only blocks auditability; container-only violates local/no-cloud and native-host measurement obligations.

Consequences: release gates cover native packages, Python packages, and source; container behavior cannot become the sole supported execution path.

## D-08 — Locked and offline-restorable dependencies

Decision: committed lockfiles and offline-restorable dependency archives are required for acceptance. Bit-for-bit artifacts are pursued where supported; otherwise normalized provenance differences are recorded.

Rejected alternatives: unlocked or network-required restoration was rejected because long-term evidence would depend on mutable external state.

Consequences: dependency/codegen versions, SBOMs, source archives, and clean offline rebuilds become release evidence; reproducibility differences remain explicit.

## Supersession

Only a dated, user-approved PSPR addendum and replacement ADR may relax distribution or offline-restoration obligations.
