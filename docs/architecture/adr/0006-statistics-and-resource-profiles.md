# ADR 0006: Statistics and resource profiles

- Status: accepted
- Date: 2026-07-18
- Owner: BONSAI maintainers
- Source: approved PSPR v0.1

## D-15 — Statistical evidence

Decision: smoke uses one deterministic seed; conformance uses five paired seeds; C2/C3 candidates use at least 20 paired seeds; C4/C5 use at least 30 paired seeds across at least three relevant scenario families. Primary outcomes are preregistered, paired 95% bootstrap intervals and effect sizes are reported, and Holm correction applies to a declared family.

Rejected alternatives: lower counts without power analysis and uncorrected multiple comparisons were rejected as insufficient for stronger claims.

Consequences: failed seeds remain visible; reduced designs require approved power analysis and cannot inherit stronger eligibility.

## D-16 — Canonical resource profiles

Decision: S is 2,000 steps, one seed, two minutes, 1 GiB agent RSS, 64 MiB agent storage, and 512 MiB observer output. C is 100,000 steps, five seeds, 60 minutes per seed, 2 GiB RSS, 256 MiB agent storage, and 5 GiB observer output. A is 1,000,000 steps, 30 seeds, eight hours per seed, 4 GiB RSS, 1 GiB agent storage, and 25 GiB observer output. L is at least 72 hours and 10 million steps on three paired seeds per required physical OS. Per-step CPU and action deadlines are manifest fields matched within comparisons.

Rejected alternatives: silently resizing profiles or matching only wall time were rejected.

Consequences: pilots may support a reviewed amendment before first C/A execution; orchestration must fail closed or report violations against the recorded profile.

## Supersession

Only a dated, user-approved PSPR addendum and replacement ADR, with power/resource consequences, may change these thresholds.
