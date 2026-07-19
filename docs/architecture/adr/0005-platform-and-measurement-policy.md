# ADR 0005: Platform and measurement policy

- Status: accepted
- Date: 2026-07-18
- Owner: BONSAI maintainers
- Source: approved PSPR v0.1

## D-10 — Required host matrix

Decision: first-release physical hosts are Windows 11 x86_64/MSVC, Apple-silicon macOS, and x86_64 Linux with cgroup v2. Hosted CI covers all three OS families and x86_64 macOS when capacity exists. ARM64 Windows/Linux are portability targets.

Rejected alternatives: reducing the physical matrix violates first-class OS coverage; expanding it before v1 disperses backend and acceptance work.

Consequences: CI and physical evidence remain distinct; platform availability is feature-detected and recorded.

## D-11 — Time, loss, and overhead

Decision: monotonic durations are integer nanoseconds plus measured effective clock resolution. Required-event loss is zero. The paired 95% CI upper bound must be at most 5% for median-throughput overhead and 10% for p95-action-latency overhead.

Rejected alternatives: floating or unitless durations, tolerated required-event loss, and unbounded instrumentation were rejected.

Consequences: clock resolution and paired overhead experiments are required; failure blocks or triggers reviewed rescoping rather than hidden sampling loss.

## D-12 — Compute proxies and missingness

Decision: when direct counters are unavailable, canonical proxies are process CPU time, monotonic wall time, environment steps, agent updates, parameter touches, event/work-item counts, model calls, planning backups, and declared estimated operations. Missing counters are unavailable, never zero.

Rejected alternatives: undeclared FLOP estimates and zero substitution were rejected as invalid matched-budget evidence.

Consequences: every measure carries method, unit, availability, and precision; claim rules qualify or refuse missing hard counters.

## D-13 — Accelerator scope

Decision: v1 supports NVIDIA through NVML when present and Apple integrated-GPU/system evidence only through a documented supported collector. AMD/Intel discrete collectors are parked; their devices remain runnable at E0 with non-energy proxies.

Rejected alternatives: claiming all vendors without implemented and calibrated backends was rejected.

Consequences: collectors are opt-in and capability-detected; unsupported or permission-denied states stay explicit; adding vendors needs backend and physical-host evidence.

## D-14 — Energy claim tiers

Decision: E0 permits no energy claim; E1 permits qualified within-machine estimates; E2 is required for cross-configuration energy comparison; E3 is reserved for laboratory-grade claims. A total-resource-positive C5 verdict requires at least E2 on the claiming platform.

Rejected alternatives: lower tiers for cross-configuration claims risk false precision; universal E3 would block ordinary hosts.

Consequences: energy tier constrains claim eligibility; calibration, provenance, uncertainty, and synchronization are evidence fields.

## Supersession

Only a dated, user-approved PSPR addendum and replacement ADR may change required hosts, overhead ceilings, proxies, accelerator scope, or energy eligibility.
