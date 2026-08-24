# BONSAI

**Benchmark for Online, Nonstationary, Single-pass Agent Intelligence**

BONSAI is an independent, algorithm-neutral measurement and external resource-governance instrument for online, nonstationary, single-pass agents. It observes OaK-style discovery cycles; it is not an implementation of unpublished Oak Lab algorithms.

## Governance status

- Research charter: approved v0.1 on 2026-07-18.
- PSPR: approved v0.1 on 2026-07-18; M2 useful-abstraction slice approved v0.2 on 2026-08-23; M3 cross-platform governed science approved v0.3 on 2026-08-24.
- Current execution authorization: approved PSPR v0.3 M3 roster authorized by full `run M3 STS` on 2026-08-24 (PT); prior M2-science STS does not cover this work.
- Implementation claims: M0–M2 remain complete through BV-05; M3 implements live OS backends, energy tiers, enforcement/schedulers, scenario families, and C4/C5 rules through BV-10. Hosted or container capability is not physical-host Job Object/cgroup/Apple qualification. Energy stays E0/E1 unless a collector is qualified. C4/C5 rules are executable and do not require a pass. No instrument-completion or OaK reproduction claim.
- Repository visibility: public under the approved 2026-07-18 repository-target addendum.
- License: `MIT OR Apache-2.0` at the recipient's option.

The approved PSPR is the execution roster, but approving or editing it is not authorization to execute work. Implementation may begin only after the user says `run it STS`, `run M0 STS`, or explicitly authorizes named prompts. External publication, visibility changes, privileged collectors, credentials, and destructive actions require their own authority.

## Repository identity

- Authoritative repository: `https://github.com/USS-Parks/BONSAI-Research-Labs`
- Authoritative local root: `C:\Users\17076\Documents\Reinforcement Learning Project`
- Default branch: `main`
- M0 STS branch: `codex/m0-governed-foundation`
- Parent repository history: excluded

The continuing STS session runs in an isolated Git worktree. The authoritative checkout and the isolated worktree share this repository's object database but never share a Git index.

## Sources of truth

Start with the [source-of-truth governance](./docs/governance/SOURCE-OF-TRUTH.md), [public-repository addendum](./docs/governance/addenda/2026-07-18-public-repository-target.md), and the [BONSAI Research Charter package](./BONSAI%20Research%20Charter/README.md), including the [research charter](./BONSAI%20Research%20Charter/BONSAI-RESEARCH-CHARTER.md), [OaK evidence register](./BONSAI%20Research%20Charter/OAK-EVIDENCE-AND-TRACEABILITY.md), and [approved PSPR](./BONSAI%20Research%20Charter/BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md).

## Baseline identity record

The pre-STS reconciliation on 2026-07-18 established:

- authoritative checkout top level: `C:/Users/17076/Documents/Reinforcement Learning Project`;
- baseline revision: `7d0ab846e46a9f38c3bd017da4837bf254b76bdc`;
- `main` and `origin/main` both resolved to that revision;
- the working tree was clean;
- the index contained only the six approved charter-package documents;
- no indexed path referred to `C:\Users\17076` or any parent repository;
- the isolated M0 worktree was created from that exact revision.

Prompt-level commands and results are retained in the verification log once BG-06 establishes it.

## Development checks

The foundation pins Rust 1.96.0 and supports Python 3.12 through 3.14 with dependencies locked by uv 0.11.29. From the repository root, run:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo xtask schema-check
uv run ruff check .
uv run pyright
uv run pytest
```

These are repository gates. Passing them is not evidence of physical behavior, instrument completion, or an evaluated-agent claim.

The versioned storage layout is specified in the [event segment format](./docs/architecture/EVENT-SEGMENT-FORMAT.md), [portable bundle index and blob format](./docs/architecture/BUNDLE-INDEX-AND-BLOBS.md), [analytical derivation contract](./docs/architecture/ANALYTICAL-DERIVATION-CONTRACT.md), and [bundle validation and migration contract](./docs/architecture/BUNDLE-VALIDATION-AND-MIGRATIONS.md).

The runtime boundary is specified in the [adapter protocol](./docs/architecture/ADAPTER-PROTOCOL.md), [bounded process transport](./docs/architecture/PROCESS-TRANSPORT.md), [event ingestion validation](./docs/architecture/EVENT-INGESTION-VALIDATION.md), [event partial-order semantics](./docs/architecture/EVENT-ORDERING.md), [run lifecycle and recovery contract](./docs/architecture/RUN-LIFECYCLE-AND-RECOVERY.md), and [agent/observer isolation contract](./docs/architecture/AGENT-OBSERVER-ISOLATION.md). Platform-neutral measurement begins with the [resource sample interface](./docs/architecture/RESOURCE-SAMPLE-INTERFACE.md), [clock calibration/deadline basis](./docs/architecture/CLOCK-CALIBRATION-AND-DEADLINES.md), [portable resource accounting](./docs/architecture/PORTABLE-RESOURCE-ACCOUNTING.md), and [measurement calibration harness](./docs/architecture/MEASUREMENT-CALIBRATION.md). External governance begins with [typed budget arithmetic and scopes](./docs/architecture/BUDGET-ARITHMETIC-AND-SCOPES.md), [deterministic admission decisions](./docs/architecture/GOVERNOR-ADMISSION-DECISIONS.md), the [violation lifecycle](./docs/architecture/GOVERNOR-VIOLATION-LIFECYCLE.md), the [basic supervised budget loop](./docs/architecture/BASIC-SUPERVISED-BUDGET-LOOP.md), [work-class allocation/reservation](./docs/architecture/WORK-CLASS-ALLOCATION-AND-RESERVATION.md), and the [agent storage and replay guard](./docs/architecture/AGENT-STORAGE-AND-REPLAY-GUARD.md).

M2 runtime lineage begins with the deterministic [artifact lifecycle registry](./docs/architecture/ARTIFACT-LIFECYCLE-REGISTRY.md). It preserves immutable versions, provenance, consumers, cost and utility histories, and terminal dispositions without deciding scientific utility.

The read-only [lineage graph and query layer](./docs/architecture/LINEAGE-GRAPH-AND-QUERIES.md) exposes exact ancestry, descendants, revision semantics, consumers, utility sources, and availability-preserving cost rollups without inferring causality.

The [observer-only replay analyzer](./docs/architecture/OBSERVER-ONLY-REPLAY.md) deterministically regenerates provenance-bound metric tables and reports while denying every route back to agent input and forcing indeterminate track eligibility on any attempted boundary crossing.

M3 platform backends begin with [Windows Job Object](./docs/architecture/WINDOWS-JOB-OBJECT.md), [macOS measurement](./docs/architecture/MACOS-RESOURCE-BACKEND.md), [Linux cgroup v2](./docs/architecture/LINUX-CGROUP-BACKEND.md), the [NVIDIA collector](./docs/architecture/NVIDIA-COLLECTOR.md), [energy tiers](./docs/metrics/ENERGY-TIERS.md), [energy qualification](./docs/architecture/ENERGY-QUALIFICATION.md), and [resource equivalence](./docs/metrics/RESOURCE-EQUIVALENCE.md). Governance continues with the [enforcement bridge](./docs/architecture/PLATFORM-ENFORCEMENT-BRIDGE.md), [decision replay](./docs/architecture/GOVERNANCE-DECISION-REPLAY.md), [scheduler contract](./docs/architecture/SCHEDULER-CONTRACT.md), [dense](./docs/architecture/DENSE-SCHEDULER.md) and [event-driven](./docs/architecture/EVENT-SCHEDULER.md) schedulers, and [matched-budget conformance](./docs/architecture/SCHEDULER-MATCHED-BUDGET.md). Families and freeze live in [scenario families](./docs/reference/SCENARIO-FAMILIES.md), [ablation orchestration](./docs/reference/ABLATION-ORCHESTRATION.md), and the [preregistration freeze](./docs/reference/PREREGISTRATION-FREEZE.md). [Metric reproducibility](./docs/metrics/METRIC-REPRODUCIBILITY.md), [C4/C5 adjudication](./docs/claims/C4-C5-ADJUDICATION.md), [comparison views](./docs/reporting/COMPARISON-VIEWS.md), [cross-platform equivalence](./docs/architecture/CROSS-PLATFORM-EQUIVALENCE.md), and [instrumentation overhead](./docs/architecture/INSTRUMENTATION-OVERHEAD.md) close the M3 claim path without requiring a C4/C5 pass.

The [adapter runtime conformance suite](./docs/architecture/ADAPTER-CONFORMANCE.md) gives third-party adapters deterministic protocol, isolation, ordering, lifecycle, repeatability, timeout, and track verdicts while explicitly excluding scientific-quality certification.

Analysis begins with the [versioned metric registry](./docs/metrics/REGISTRY.md), [primary behavior metrics](./docs/metrics/PRIMARY-BEHAVIOR-METRICS.md), [resource/overhead metrics](./docs/metrics/RESOURCE-AND-OVERHEAD-METRICS.md), [continual-learning metrics](./docs/metrics/CONTINUAL-LEARNING-METRICS.md), [feature metrics](./docs/metrics/FEATURE-METRICS.md), [subproblem metrics](./docs/metrics/SUBPROBLEM-METRICS.md), [option metrics](./docs/metrics/OPTION-METRICS.md), [model and knowledge metrics](./docs/metrics/MODEL-AND-KNOWLEDGE-METRICS.md), [planning metrics](./docs/metrics/PLANNING-METRICS.md), the [utility estimator hierarchy](./docs/metrics/UTILITY-ESTIMATOR-HIERARCHY.md), [discovery-cycle health](./docs/metrics/DISCOVERY-CYCLE-HEALTH.md), [failure-criteria detectors](./docs/metrics/FAILURE-CRITERIA.md), and [statistical aggregation](./docs/metrics/STATISTICAL-AGGREGATION.md); derived report values must originate in deterministic metric tables.

Reference experiments begin with the [scenario protocol](./docs/architecture/SCENARIO-PROTOCOL.md), its observer-only diagnostic channel, the [primitive control adapter](./docs/reference/PRIMITIVE-CONTROL-ADAPTER.md), the [BRDC-1 specification](./docs/reference/BRDC-1-SPEC.md), the [BRDC-1 feature and subproblem stages](./docs/reference/BRDC-1-FEATURE-SUBPROBLEM.md), the [BRDC-1 option and model stages](./docs/reference/BRDC-1-OPTION-MODEL.md), [BRDC-1 planning and credit](./docs/reference/BRDC-1-PLANNING-CREDIT.md), [BRDC-1 curation](./docs/reference/BRDC-1-CURATION.md), and [BRDC-1 comparators](./docs/reference/BRDC-1-COMPARATORS.md).

Concrete [C0 and C1 adjudication](./docs/claims/C0-C1-ADJUDICATION.md) assigns exact verdicts to compliant, soft-degraded, hard-violating, unavailable, tampered, and ambiguous-track bundles. [C2 and C3 adjudication](./docs/claims/C2-C3-ADJUDICATION.md) requires continual adaptation and exact or matched abstraction utility; proxy-only utility and track leakage cannot pass.
