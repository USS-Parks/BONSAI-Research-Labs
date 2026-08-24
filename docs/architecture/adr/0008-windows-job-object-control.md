# ADR 0008: Windows Job Object control spike

- Status: accepted
- Date: 2026-08-24
- Owner: BONSAI maintainers
- Source: approved PSPR v0.3 BM-06

Windows hard limits are Job Object CPU, memory, process count, and I/O where the host probe can attach. Kill-on-governor-close is required when the job is attached. This crate records capability and fixture enforcement without a safe Job Object binding.

This is not a hostile sandbox. D-21 is unchanged.

## Supersession

Only a dated, user-approved PSPR addendum and replacement ADR may add an unsafe or privileged Job Object binding.
