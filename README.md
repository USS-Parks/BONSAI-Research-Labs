# BONSAI

**Benchmark for Online, Nonstationary, Single-pass Agent Intelligence**

BONSAI is an independent, algorithm-neutral measurement and external resource-governance instrument for online, nonstationary, single-pass agents. It observes OaK-style discovery cycles; it is not an implementation of unpublished Oak Lab algorithms.

## Governance status

- Research charter: approved v0.1 on 2026-07-18.
- PSPR: approved v0.1 on 2026-07-18.
- Current execution authorization: remaining approved PSPR roster in dependency order, expanded by `Continue to STS` on 2026-07-18.
- Implementation claims: M0 governed foundation, BC-01 through BC-12, BR-01 through BR-06, BM-01 through BM-04, BQ-01 through BQ-04, and BK-01 through BK-02 are complete; no instrument-completion or C0–C5 claim.
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

The runtime boundary is specified in the [adapter protocol](./docs/architecture/ADAPTER-PROTOCOL.md), [bounded process transport](./docs/architecture/PROCESS-TRANSPORT.md), [event ingestion validation](./docs/architecture/EVENT-INGESTION-VALIDATION.md), [event partial-order semantics](./docs/architecture/EVENT-ORDERING.md), [run lifecycle and recovery contract](./docs/architecture/RUN-LIFECYCLE-AND-RECOVERY.md), and [agent/observer isolation contract](./docs/architecture/AGENT-OBSERVER-ISOLATION.md). Platform-neutral measurement begins with the [resource sample interface](./docs/architecture/RESOURCE-SAMPLE-INTERFACE.md), [clock calibration/deadline basis](./docs/architecture/CLOCK-CALIBRATION-AND-DEADLINES.md), [portable resource accounting](./docs/architecture/PORTABLE-RESOURCE-ACCOUNTING.md), and [measurement calibration harness](./docs/architecture/MEASUREMENT-CALIBRATION.md). External governance begins with [typed budget arithmetic and scopes](./docs/architecture/BUDGET-ARITHMETIC-AND-SCOPES.md), [deterministic admission decisions](./docs/architecture/GOVERNOR-ADMISSION-DECISIONS.md), the [violation lifecycle](./docs/architecture/GOVERNOR-VIOLATION-LIFECYCLE.md), and the [basic supervised budget loop](./docs/architecture/BASIC-SUPERVISED-BUDGET-LOOP.md).

Analysis begins with the [versioned metric registry](./docs/metrics/REGISTRY.md) and [primary behavior metrics](./docs/metrics/PRIMARY-BEHAVIOR-METRICS.md); derived report values must originate in deterministic metric tables.
