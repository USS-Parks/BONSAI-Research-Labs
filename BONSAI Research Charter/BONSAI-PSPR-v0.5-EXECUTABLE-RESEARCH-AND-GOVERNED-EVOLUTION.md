# BONSAI PSPR v0.5 — Executable Research and Governed Evolution

Version: 0.5 (approved from 0.5-draft.1)\
Date: 2026-09-05 (America/Los_Angeles)\
Status: **APPROVED — STS AUTHORIZED FOR BX-01–BX-40**\
Initiative: Complete the executable research loop, prove adapter flexibility, and support measured evolution under stable external authority.\
Authoritative implementation root: **C:\Users\17076\Documents\Reinforcement Learning Project**\
Build repository: **USS-Parks/BONSAI-Research-Labs**, branch main\
Plan owner and approver: Basho Parks\
Draft author: Codex

> Historical drafting boundary (superseded for execution by the approval in section 14): This request authorizes drafting only. No implementation, checkout update, source commit, push, new worktree, host reconfiguration, or experiment has been performed for this draft. Approval of the document does not start STS. Execution requires an explicit instruction such as “run M5a STS,” “run PSPR v0.5 STS,” or authorization of named BX prompts. Earlier M2–M4 STS phrases do not authorize this roster.

## 0. Governance and source of truth

### 0.1 Mission and scope

BONSAI remains an algorithm-neutral observer, evidence producer, and external resource governor. Its purpose is to determine whether an online, nonstationary, single-pass agent develops reusable state and temporal abstractions whose downstream benefit exceeds their acquisition and maintenance costs. It is not a reproduction of unpublished Oak Lab algorithms.

This follow-on plan turns inspected scaffolding and interfaces into runnable, falsifiable experiments. It retains the Rust/Python stack and established wire/storage formats. It adds reliable execution, causal environments, evidence construction, bounded lineage processing, substantive BRDC-1 behavior, independent adapters, all ten charter scenario families and eleven ablation pairs, controlled candidate evolution, and physical/long-duration verification.

Instrument capability and evaluated-agent success remain separate. A correctly executed experiment may return FAIL or INDETERMINATE. No prompt requires manufacturing a positive C4/C5 result.

### 0.2 Authority and location

The user's current instructions and approved addenda take precedence, followed by the research charter and OAK-EVIDENCE-AND-TRACEABILITY, approved PSPRs/ADRs/contracts, and the execution/evidence ledgers. Inspect code and actual test evidence to establish what behavior exists; a completed historical checkbox does not establish a stronger untested claim.

The charter-only USS-Parks/BONSAI repository is not the implementation remote. Do not publish source there.

This review draft is stored alongside the architecture review in this task's outputs folder. Upon approval, the proposed canonical plan location is:

    BONSAI Research Charter/BONSAI-PSPR-v0.5-EXECUTABLE-RESEARCH-AND-GOVERNED-EVOLUTION.md

BX-01 will adopt the approved version there and add its handoff/reference entries. It must not maintain competing editable copies. The review copy becomes an identified historical draft.

### 0.3 Verified baseline and remaining uncertainty

| Item | State at drafting |
|---|---|
| Canonical local HEAD | 3ad391bc3118cb0c956b394671f54ab1dd308fe5; main; clean tracked/untracked porcelain status |
| Cached origin/main | 2cdd26d043ce8e53bb983e0778cbc0de952f9c30; local branch appears only three behind this stale ref |
| Live GitHub main | 2f35355c6d12d019eb8625cb3bd38728d90ee029; rechecked with ls-remote for this draft; nine commits ahead of local HEAD |
| Historical M3/M4 | DEVLOG records closure under explicitly accepted capability/physical-host exceptions. Preserve those records. |
| Physical acceptance | Required real-host controls, clean restore, energy qualification, and L evidence are not established by this review; M4 records 72-hour physical runs as not-run. |
| Architecture-review testing | 27 selected Python tests passed against local HEAD; not a full suite or a live-main run. |
| Source review | Critical inspected files were compared with the intervening remote delta. Recent preflight/checksum fixes must not be reimplemented. BX-01 revalidates every finding at the execution baseline. |

Do not blindly pull into a changed checkout. BX-01 must recheck staged, unstaged, untracked, local-only, and concurrent work before a safe fast-forward. If the branch diverges, preserve it and resolve the exact discrepancy before implementation. Record the immutable execution SHA; “latest main” alone is insufficient provenance.

### 0.4 Relationship to approved history

v0.1 remains the original charter-to-prompt map; v0.2–v0.4 remain historical approved slices. This document proposes a **new follow-on milestone M5**, divided into independently approvable cuts M5a–M5f. It does not assert that M0–M4 never happened, erase their gates, or silently change their recorded exceptions.

The new **BX-01–BX-40** identifiers distinguish follow-up implementation and stronger evidence gates from earlier BG/BC/BR/BM/BQ/BK/BE/BV prompts. Each BX prompt names its legacy mapping. On approval, this is an explicit additive roster; v0.3's requirement to preserve original IDs for its own remaining M3 work is not changed retroactively.

Append an evidence-gap reconciliation table in BX-01 with columns: historical prompt and decision, recorded result, behavior observed now, outstanding gate, replacement/extension BX ID, and affected claims. Leave historical execution records intact. Any necessary ADR or contract supersession must name the prior decision and migration implications.

### 0.5 Execution and publication discipline

- Run one approved prompt at a time in dependency order. No parallel agent or worktree lane is implied.
- One prompt maps to one focused implementation commit. If a prompt proves too large, split it into reviewed stable suffixes before executing it; do not repeat the earlier 29-prompt or six-prompt bundles as a convenience.
- A prompt becomes complete only after its prescribed gates and DEVLOG/verification entries exist. Record source/commit identity without impossible self-referential commit hashes; use the established subsequent closeout-entry convention.
- The 2026-07-19 continuing-source-publication addendum remains applicable after fresh execution authorization: gated source prompts may be fast-forward-pushed to Labs main, with exact remote SHA and required hosted CI evidence recorded. No repetitive source-publication approval is needed within that authority. No force-push or upload of unredacted host data is authorized.
- Package uploads, release tags, benchmark submissions, marketing/scientific announcements, equipment purchases, privileged host changes, and parked-scope revival retain their separate existing authorization requirements. This draft supplies none of them.
- When a host-dependent gate cannot run, record the exact blocker and evidence state. Do not convert not-run to pass. Pause dependent work; independent continuation requires an already approved dependency order or an explicit deviation. If the user defers a physical gate, record that exception and continue authorized independent work without claiming full milestone completion.

## 1. Settled architecture and proposed defaults

Existing D-01–D-21 and approved addenda remain in force. The following specifies how this roster applies them; items marked proposed become settled only when approved.

| Decision | Default and override boundary |
|---|---|
| Stack | Retain Rust for contracts, transport, accounting, governance, durable evidence, and core analysis; Python for reference science and adapters. No rewrite. |
| Protocol and storage | Retain versioned length-delimited Protobuf child-process protocol; immutable manifests/events/blobs; SQLite index; derived Arrow/Parquet. Use existing migration and compatibility machinery. |
| Hot execution path | Keep plotting, HTML generation, and heavyweight table materialization outside the action-critical path. Extract existing functionality only where dependency/overhead measurements justify it. |
| Algorithm neutrality | The supervisor must not know BRDC-1 classes or hard-code its policy. Prove reuse with a materially different agent, rather than a renamed comparator. |
| Initial execution host — proposed | Ubuntu/Linux, including a correctly identified WSL instance for development integration, is the first executable lane. A WSL/container result is never relabeled physical Linux acceptance. Windows 11 x86_64 and Apple-silicon macOS remain required targets. |
| Runtime safety | Fault containment and bounded execution under D-21; no hostile-code sandbox claim. Keep workspace unsafe-code restrictions. Use safe established bindings or a narrowly reviewed helper boundary if native API access requires it; any policy exception needs an explicit ADR. |
| Self-evolution — proposed | Within-run discovery and curation plus between-run versioned candidate selection. Default candidate mutations are declared parameter/representation structures, checkpoints, and pre-reviewed adapter versions. No arbitrary generated native programs, evaluator edits, unattended self-deployment, or privileged candidate execution. |
| Selection semantics — proposed | Between-run optimizer access to evaluation scores is meta-level search, not evidence that one Track A learner obtained those scores through single-pass experience. Keep fresh held-out evaluation, search lineage, and full search-cost accounting separate from within-run online credit. |
| Resource caps | D-16 profiles unchanged. A common outer safety ceiling remains active even for the “unconstrained growth” scientific comparator; disabling the experimental governor never disables the supervisor's machine-protection ceiling. Censored comparisons are labeled ineligible, not silently normalized. |
| Modernity | Keep reproducible experiment pins. BX-38 audits current supported releases/advisories, tests upgrades and the declared Python range, and records compatibility. Do not preselect a future “latest” dependency version in this draft. |
| External environment reuse | Gymnasium support remains an optional adapter, not a core dependency or replacement for BONSAI-owned causal diagnostics. |

No blocking preference question is required to review this draft. The user may narrow the first host, milestone scope, or evolution boundary on approval. Changes to evidence thresholds, track semantics, or required physical targets require a named addendum, not an implementation-time shortcut.

## 2. Verification gates — apply before the roster

### 2.1 Gate vocabulary and evidence states

Every BX prompt must satisfy its own acceptance criteria plus the applicable universal gates below. Record **PASS**, **FAIL**, **INDETERMINATE**, **NOT-RUN**, or **BLOCKED** explicitly. A negative scientific result can accompany a passing implementation gate if the protocol and analysis are correct. A missing test or host cannot.

| Gate | Required proof |
|---|---|
| G1 — Repository | Focused behavior tests plus applicable locked repository checks; exact commands/output, source identity, timestamps, exit codes, artifact hashes, and host/runner fingerprint. |
| G2 — Live execution | Actual agent/environment subprocess interaction, OS resource facts, and observed termination/cleanup for the capability claimed. Mock records and synthetic booleans do not close this gate. |
| G3 — Evidence integrity | Derived values can be independently reconstructed from immutable input bundles; altered/missing inputs, invalid lineage, forbidden information exposure, and inconsistent track metadata prevent stronger claims. |
| G4 — Scientific validity | Preregistered causal intervention, matched conditions, correct seed pairing, declared metric/analysis versions and uncertainty, held-out confirmatory runs, and honest comparator eligibility. |
| G5 — Boundedness/recovery | Enforced process and I/O deadlines; bounded queues and observer working state; measured growth; partial-write recovery; no survivor processes or invented completion after interruption. |
| G6 — Compatibility | Current/previous compatible schema and adapter fixtures, explicit epoch refusal/migration for breaking changes, deterministic semantic checks distinct from machine-dependent timings. |
| G7 — Physical acceptance | Attested required physical hosts, measured controller behavior and available counters, actual long-duration evidence, clean restore, and overhead gates; hosted/WSL runs remain separately labeled. |

**Existing commands:** use the applicable subset of cargo fmt --all --check; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo xtask schema-check; uv run --frozen ruff check .; uv run --frozen pyright; uv run --frozen pytest; and the actual scripts/check_docs.py, check_adrs.py, check_license.py, check_governance_ledgers.py, check_terminology.py, and check_ci.py controls.

Use existing cargo xtask verify for execution records and cargo xtask bundle-check for produced bundles. The original plan's proposed “evidence-check” command is not present in the inspected xtask; do not report it as an executed command. New scenario/run commands are proposed deliverables until implemented and documented. Offline restore and RC checks must prove restoration from an intentionally clean environment at acceptance, not merely that a warmed cache can run cargo test --offline.

Run relevant checks per prompt and the complete repository/publication gate before an authorized source push. Do not add tests that merely mirror record constructors or check filenames. Required boundary tests use live subprocesses/OS effects; physical claims use physical machines. No silent xfail, quarantine, disabled gate, weakened timeout, or reduced sample count is allowed.

### 2.2 Scientific and resource thresholds carried forward

| Item | Required baseline |
|---|---|
| Smoke S | 2,000 steps; one seed; two-minute wall cap; 1 GiB agent RSS; 64 MiB agent storage; 512 MiB observer output. |
| Conformance C | 100,000 steps; five paired seeds; 60 minutes per seed; 2 GiB RSS; 256 MiB agent storage; 5 GiB observer output. |
| Acceptance A | 1,000,000 steps; 30 paired seeds; eight hours per seed; 4 GiB RSS; 1 GiB agent storage; 25 GiB observer output. |
| Long L | At least 72 hours **and** 10 million steps on each of three paired seeds per required physical OS. Meeting only time or steps is insufficient. Freeze exact host/resource allocations before launch. |
| C2/C3 candidate claims | At least 20 paired seeds; canonical prerequisites; preregistered primary outcomes, paired 95% bootstrap intervals, effect sizes, Holm correction across the declared comparison family. |
| C4/C5 candidate claims | At least 30 paired seeds across at least three relevant scenario families, plus the canonical lifetime/generation/ablation prerequisites. Passing only smoke/conformance does not qualify. |
| Overhead | Upper bound of paired 95% CI: at most 5% median-throughput overhead and at most 10% p95 action-latency overhead. Required-event loss is zero. |
| Energy | Missing is unavailable, never zero. E0: no energy claim; E1: qualified within-machine estimate; E2 required for cross-configuration energy comparisons and C5 total-resource-positive claims; E3 lab claims remain parked. |
| Counterfactual utility | Exact omit-artifact/event diagnostics or matched ablations; proxy credit alone cannot establish C3. Consequential planning uses identical pre-backup state/random draws and the declared policy/action divergence criterion and attribution horizon. |

Seed pairing means shared exogenous randomness and initial conditions where causally appropriate. It does **not** mean forcing identical observation trajectories after different actions in an action-dependent world. Track A/B/C/D classification is based on enforced information access and update facts, not a config label. Threshold changes need an explicit reviewed amendment before confirmatory data collection.

## 3. Milestones and approval cuts

| Cut | Prompts | Usable result and milestone gate |
|---|---|---|
| **M5a — One real governed run** | BX-01–BX-06 | On the initial Linux lane, a real closed-loop primitive learner/environment run emits independently verified C0/C1 evidence; actual limit, cancellation, failure, and track behavior are demonstrated. |
| **M5b — Bounded and reusable instrument** | BX-07–BX-11 | Incremental lineage and crash recovery work under bounded state; a second materially different agent and optional external environment use the same versioned boundary. |
| **M5c — Executable discovery comparisons** | BX-12–BX-20 | BRDC-1 learns and revises artifacts during interaction; all eleven ablations launch real treatments; bundle-derived C2–C5 decisions work without requiring positive research outcomes. |
| **M5d — Ten causal families** | BX-21–BX-30 | Ten diagnostic mechanisms and enlarged counterparts demonstrate their named causal properties; the runner can select each through the same interface. |
| **M5e — Governed evolution** | BX-31–BX-33 | Candidate ancestry, independent search accounting, held-out promotion, checkpoint compatibility, and rollback are demonstrated across generations. |
| **M5f — Qualified acceptance and handoff** | BX-34–BX-40 | Required physical platforms, overhead, supported toolchains, actual A/L runs, offline restore, and reproducible operator handoff are completed or precisely identified as remaining blockers. M5f cannot be fully complete while a mandatory gate is missing. |

Recommended first authorization: **M5a only**, yielding a usable experimental instrument before expanding the science. Full v0.5 STS, if explicitly granted, follows the same sequence and does not waive host/privilege/publication restrictions.

## 4. Sequential prompt roster

Initial drafting state: all prompts were **[ ] NOT STARTED**. Current execution state is recorded in the headings and section 14. Dependencies are explicit; the numbered order is the default sequential roster. Each prompt inherits §2 and produces its DEVLOG/verification closeout. “Evidence” names the minimum retained artifact, not merely a document that asserts a pass.

### [x] BX-01 — Adopt the approved baseline and reconcile evidence gaps

**Milestone:** M5a. **Depends:** Approved v0.5 scope and fresh STS authorization.\
**Legacy mapping:** BG-01–BG-10; all M3/M4 exceptions.\
**Reuse:** Reuse governance and evidence ledgers.

**Objective:** Establish one preserved, reproducible execution baseline for this approved follow-on roster.

**Work and boundary:** Recheck checkout identity and all local work; compare the current live main and adopt it only by a safe authorized fast-forward. Read AGENTS/RTK and current PSPR/DEVLOG. Revalidate review findings, retained worktrees, available hosts and disk, actual gate commands, and source publication authority. Adopt the approved plan and a compact handoff.

**Acceptance gate:** Every review finding has a current-code disposition and legacy/BX mapping. No pre-existing work is lost. The baseline repository gate is recorded as pass or an exact blocker; unexplained failures block implementation. Record branch, immutable SHA, dirty state, host classes, resource availability, and current source/remote relationship.

**Retained evidence:** Baseline and evidence-gap matrix; exact baseline verification records; worktree/storage inventory.

### [x] BX-02 — Bound adapter I/O and process-tree cancellation

**Milestone:** M5a. **Depends:** BX-01.\
**Legacy mapping:** BR-03–BR-06; BV-12.\
**Reuse:** Extend bonsai-runtime; reuse process abstractions and OS bindings.

**Objective:** Guarantee bounded failure and cleanup for faulty adapters on the initial Linux lane.

**Work and boundary:** Own the launched process group/tree; bound writes, reads, queues, shutdown, and joining/cleanup. Propagate cancellation and record survivors/cleanup failures. Preserve protocol framing and useful error identity. Windows/macOS native integration is completed in BX-34/35.

**Acceptance gate:** Live fixtures cover a child not reading stdin, a grandchild retaining pipes, stderr flood, partial frames, normal exit, timeout, and supervisor cancellation. Each reaches a terminal result within the declared timeout plus a frozen cleanup allowance; OS inspection finds no test descendants. Repeat start/stop cycles with bounded handles/threads. A hung helper must itself be bounded by the test harness.

**Retained evidence:** Process IDs/tree membership, monotonic deadlines, captured failures, and post-test survivor/handle checks.

### [~] BX-03 — Apply and verify Linux resource authority

**Milestone:** M5a. **Depends:** BX-02.\
**Legacy mapping:** BM-09–BM-10; BQ-07.\
**Reuse:** Extend bonsai-platform Linux backend and governor bridge.

**Objective:** Make Linux limit records correspond to actual controller effects on the evaluated process tree.

**Work and boundary:** Use a delegated cgroup v2 subtree; attach the intended descendants, apply supported CPU/memory/process controls, measure counters, and verify termination/cleanup. Treat process-group cancellation separately from resource accounting. Unsupported controls block claims that require them.

**Acceptance gate:** Live workloads demonstrate actual membership, a memory-limit crossing, the declared CPU-control semantics, descendant accounting, denial/unsupported paths, and empty cleanup. Merely creating a directory or constructing an enforcement record cannot pass. No global cgroup changes; missing delegation is a blocker for this lane. Label WSL/container/physical accurately.

**Retained evidence:** Controller settings/readbacks, observed counters/violations, process cleanup, and capability matrix.

### [ ] BX-04 — Implement a stepwise causal environment boundary

**Milestone:** M5a. **Depends:** BX-03.\
**Legacy mapping:** BE-01; BC adapter contracts; D-18.\
**Reuse:** Extend existing ScenarioSpec and adapter messages.

**Objective:** Let the next transition depend on the action actually chosen during online interaction.

**Work and boundary:** Add reset/step state to the existing deterministic diagnostic environment; separate learner observations from privileged diagnostic state. Preserve deterministic seed streams, terminal versus truncated outcomes, and compatibility with archived fixtures.

**Acceptance gate:** A causal diagnostic shows that two actions at the same state can produce the prescribed different outcomes; identical initial state/seed and action sequence reproduce the semantic trace. The learner cannot receive future action schedules or diagnostic truth. Invalid/out-of-order steps and post-terminal actions fail with bounded errors.

**Retained evidence:** Golden causal traces, differential action tests, protocol compatibility fixtures, and exposure checks.

### [ ] BX-05 — Compose one end-to-end governed experiment runner

**Milestone:** M5a. **Depends:** BX-04.\
**Legacy mapping:** BR runtime; BE-03; BE-15; BV-02–BV-03.\
**Reuse:** Compose existing runtime, governor, ingestor, bundle, control, and report modules.

**Objective:** Run a real primitive learner against the causal environment and persist a usable experiment bundle.

**Work and boundary:** Introduce the smallest operator command at an existing CLI seam; add a thin CLI target only if no suitable target exists. Load an immutable manifest, launch agent/environment children, exchange actual actions, enforce policy, append required events, and produce a static report after the run. No second general orchestration framework.

**Acceptance gate:** The S-profile run completes within its declared limits and records every required action/reward/work/resource event. Forced agent failure, resource violation, storage exhaustion, and cancellation produce classified incomplete bundles without fabricated success. Existing heartbeat remains reproducible. Report generation is outside the action deadline.

**Retained evidence:** One complete S bundle, classified failure bundles, operator command, and full input/source identity.

### [ ] BX-06 — Derive C0/C1 and track eligibility from run evidence

**Milestone:** M5a. **Depends:** BX-05.\
**Legacy mapping:** BV-01; BV-04; BC-05; BR-09.\
**Reuse:** Extend existing bundle validator and C0/C1 adjudication.

**Objective:** Make the first real run independently auditable without trusting caller-supplied compliance flags.

**Work and boundary:** Build C0/C1 inputs from inventory hashes, event continuity, runtime resource facts, information-access records, and policy identity. Expose immutable verified facts to the pure decision rules. Keep unsupported/ambiguous evidence ineligible.

**Acceptance gate:** A fresh verifier process reconstructs the valid run verdict. Modified files, missing controller evidence, forbidden history exposure, policy substitution, required-event gaps, and unknown track classification cannot produce C0/C1 or stronger eligibility incorrectly. Live negative fixtures verify the actual boundary; arbitrary hostile-code isolation is not claimed.

**Retained evidence:** Independent verify outputs and positive/negative bundle corpus. This closes M5a only if BX-01–06 all pass.

### [ ] BX-07 — Make lineage validation incremental

**Milestone:** M5b. **Depends:** BX-06.\
**Legacy mapping:** BC-07; BR-07–BR-08; BK lineage consumers.\
**Reuse:** Extract incremental state from the existing contract validator.

**Objective:** Remove full-history clone/revalidation from each accepted lifecycle event.

**Work and boundary:** Use transactional validation against current state; retain replay validation as the independent oracle. Preserve rejection atomicity, revision ownership, and existing error semantics. Do not invent a second contradictory legality model.

**Acceptance gate:** Generated valid/invalid event sequences yield the same accept/reject behavior and reconstructed state as full replay. Rejected events leave state unchanged. At 1k, 10k, and 100k events, instrumentation shows a new event visits only its required relations/current state rather than the full prefix; report time/RSS and branching cases. No unsupported constant-time claim for arbitrary ancestry traversal.

**Retained evidence:** Differential/property test corpus; operation-count and scaling measurements with workload definitions.

### [ ] BX-08 — Bound persistent evidence state and recover interrupted runs

**Milestone:** M5b. **Depends:** BX-07.\
**Legacy mapping:** BC-08–BC-12; BR recovery; BK-14; BV-12.\
**Reuse:** Extend event segments, blob store, SQLite index, and checkpoint/recovery code.

**Objective:** Keep observer memory bounded by declared live state and buffers while preserving complete durable evidence.

**Work and boundary:** Stream segments and derived data, checkpoint validated lineage/index state, and retrieve historical ancestry on demand. Add crash-consistent continuation rules; interrupted sessions must not silently become uninterrupted Track A traces. Retain user data and follow declared output quotas.

**Acceptance gate:** Increasing completed history with the same live-artifact cap does not retain the full event prefix in RSS. Inject interruption before/after segment and index commits; recover an exact committed prefix, detect truncation/corruption, and classify continuation. Disk-full and quota exhaustion stop cleanly; no deletion outside the run-owned output root.

**Retained evidence:** Bounded-memory measurements, failure/recovery bundles, checkpoint compatibility fixtures, and quota records.

### [ ] BX-09 — Prove versioned extension contracts

**Milestone:** M5b. **Depends:** BX-08.\
**Legacy mapping:** BC-01–BC-06; BR-10; D-03–D-05.\
**Reuse:** Extend capability negotiation, schema epochs, and AdapterConformanceSuite.

**Objective:** Support independently evolving adapters without coupling the supervisor to one scientific implementation.

**Work and boundary:** Document required versus optional capabilities for observation/action/work/feedback/checkpoint/artifact events. Preserve immutable per-run capabilities; capability changes occur across explicit versioned run boundaries. Remove concrete BRDC-1 dependencies from the authority layer only where found.

**Acceptance gate:** Current and previous compatible adapters pass the same conformance suite; unknown optional fields are handled according to protocol, unknown required capabilities fail closed, and breaking epochs require a tested migration or refusal. Archived bundles remain verifiable. Dependency checks show no reference-agent import in the generic supervisor.

**Retained evidence:** Compatibility matrix, protocol migration fixtures, and dependency-boundary audit.

### [ ] BX-10 — Integrate a materially different agent through the same adapter

**Milestone:** M5b. **Depends:** BX-09.\
**Legacy mapping:** BE-09; BR-10; D-02; D-17.\
**Reuse:** Implement at the existing adapter seam; reuse a vetted public baseline if suitable.

**Objective:** Demonstrate algorithm flexibility with a different state representation and learning procedure.

**Work and boundary:** Select the smallest credible second baseline after a reuse/license review, such as a compact tabular transition learner alongside the linear/reference path. It must differ in representation and update logic, not just a flag or label. No neural-framework mandate.

**Acceptance gate:** Both agents execute through the same supervisor binary, manifest family, resource policies, conformance suite, and report path without an agent-specific branch in authority code. Five paired conformance seeds run on the common causal world; outputs show actual distinct update behavior and correctly separated resource usage.

**Retained evidence:** Selection rationale, adapter conformance results, and paired run bundles.

### [ ] BX-11 — Integrate an optional external environment adapter

**Milestone:** M5b. **Depends:** BX-10.\
**Legacy mapping:** BE-01; D-18.\
**Reuse:** Reuse Gymnasium API through a separately optional adapter.

**Objective:** Demonstrate environment flexibility without weakening BONSAI stream and evidence semantics.

**Work and boundary:** Wrap one deterministic, action-dependent Gymnasium environment with pinned dependencies and license/source provenance. Preserve seed handling, action/observation schema validation, termination/truncation, and explicit online-only input.

**Acceptance gate:** Direct environment execution and adapter execution agree for the same seed/action sequence, including time-limit truncation. The regular BONSAI run works with both agents; installing the optional adapter is unnecessary for core runs. No environment truth is sent to the learner outside the declared observation.

**Retained evidence:** Direct-versus-adapter traces, optional-dependency install test, and run bundles. M5b closes after BX-07–11.

### [ ] BX-12 — Discover useful feature candidates from online experience

**Milestone:** M5c. **Depends:** BX-11.\
**Legacy mapping:** BE-04; BK-04–BK-06.\
**Reuse:** Extend BRDC-1 FeatureStage and existing artifact events.

**Objective:** Replace supplied feature examples with an executable, budgeted candidate-construction mechanism.

**Work and boundary:** Specify a bounded tabular/linear feature grammar and online proposal rule. Record creation, revision, parents, evidence exposure, parameter/storage costs, and admissions. Keep the original tiny fixtures as legality tests.

**Acceptance gate:** A diagnostic stream triggers feature proposals from experience without externally supplied representations or utility labels. Identical seeds reproduce proposals; feature-disabled control changes the intended mechanism. Denied work/allocation cannot mutate state, and measured serialized/allocated size is not confused with tuple element count.

**Retained evidence:** Online proposal traces, lineage and resource records, and enabled/disabled diagnostic runs.

### [ ] BX-13 — Learn feature-attainment subproblems and executable options

**Milestone:** M5c. **Depends:** BX-12.\
**Legacy mapping:** BE-05; BK-06–BK-07.\
**Reuse:** Extend BRDC-1 option/subproblem stages.

**Objective:** Make discovered features induce learned temporally extended behavior.

**Work and boundary:** Implement reward-respecting subproblem construction and online option policy/termination updates. Route actual option initiation, primitive actions, and termination through the run protocol; charge construction and maintenance work.

**Acceptance gate:** Options learned from experience execute for variable duration, terminate correctly, and preserve the environmental reward/accounting distinction. The primitive-only and reward-oblivious controls alter only the declared mechanism. No test supplies a finished option policy as proof of learning.

**Retained evidence:** Learning curves, option execution traces, budget admissions, and comparator bundles.

### [ ] BX-14 — Learn transition and option models from observed outcomes

**Milestone:** M5c. **Depends:** BX-13.\
**Legacy mapping:** BE-06; BK-08.\
**Reuse:** Extend existing BRDC-1 model stage and metrics.

**Objective:** Produce predictive models whose learning, calibration, and drift are measurable.

**Work and boundary:** Learn appropriate state-conditioned transition/reward/duration quantities for primitive actions and options. Version model artifacts; record updates, prediction errors, model memory, and work. Avoid treating one overwritten observation as proof of a useful learned model.

**Acceptance gate:** Controlled environments have known outcomes that independent tests can compare to predictions across more than one state/action/duration. Held-out diagnostic observations quantify error and drift; unavailable predictions remain explicit. Agent learning never consumes observer-only diagnostic targets.

**Retained evidence:** Predictive checks, drift traces, model versions, and resource accounting.

### [ ] BX-15 — Execute planning and exact paired counterfactuals

**Milestone:** M5c. **Depends:** BX-14.\
**Legacy mapping:** BE-07; BK-09–BK-11; D-19–D-20.\
**Reuse:** Extend planning stage and existing utility/consequentiality definitions.

**Objective:** Measure whether an actual planning backup changes downstream decisions.

**Work and boundary:** Run budgeted backups from learned models; capture compatible pre-backup state/randomness and execute omit-one-backup/artifact diagnostic branches outside learner access. Charge or separately account for evaluator counterfactual work under the declared protocol.

**Acceptance gate:** An independent exact diagnostic reproduces both consequential and inconsequential backups using the D-19 criterion. The test computes realized/counterfactual actions rather than supplying their difference. Snapshot mismatch, differing random draws, and uncharged/hidden evaluator work invalidate the comparison.

**Retained evidence:** Paired branch identities, model/planning traces, exact utility results, and timing/work ledgers.

### [ ] BX-16 — Compute slower backward utility credit

**Milestone:** M5c. **Depends:** BX-15.\
**Legacy mapping:** BE-07; BK-11–BK-13.\
**Reuse:** Extend the backward-credit stage and metric derivations.

**Objective:** Propagate measured downstream contribution to the correct earlier artifacts.

**Work and boundary:** Use exact diagnostic utility to calibrate the online estimator and its declared delay/attribution horizon. Track consumed observations and versions; online learner feedback must remain within the authorized signal contract. Label approximations separately from exact evidence.

**Acceptance gate:** Known positive, redundant, stale, and negative-contribution cases receive the expected relative attribution without directly supplied utility labels. Delayed outcomes credit the correct artifact revision; ancestor sharing does not silently double-count benefit. Proxy-only evidence cannot pass C3.

**Retained evidence:** Exact-versus-estimated calibration, attribution traces, and counterexample fixtures.

### [ ] BX-17 — Drive curation from measured credit under budgets

**Milestone:** M5c. **Depends:** BX-16.\
**Legacy mapping:** BE-08; BQ-05–BQ-06.\
**Reuse:** Extend lifecycle proposals, governor admissions, and BRDC-1 curation.

**Objective:** Make the agent retain, revise, retire, and replace artifacts during a real run.

**Work and boundary:** Apply the declared utility estimator and bounded maintenance schedule to active artifacts. Preserve historical identities and dependencies. The governor retains final allocation authority; revisions emit actual versioned events.

**Acceptance gate:** During interaction, resource pressure and changed utility trigger traceable retain/replace/retire decisions. Denied changes do not alter live state. Referenced artifacts cannot be silently removed; cleanup reconciles memory/storage accounting. Frozen-representation and random/age-retention controls demonstrate the intended differences.

**Retained evidence:** Lifecycle and admission traces, bounded-state measurements, and comparator bundles.

### [ ] BX-18 — Compare real dense and event-driven scheduling

**Milestone:** M5c. **Depends:** BX-17.\
**Legacy mapping:** BQ-09–BQ-12.\
**Reuse:** Extend existing scheduler implementations and work accounting.

**Objective:** Execute a valid scheduler comparison without conflating scheduling with batch size.

**Work and boundary:** Drive identical eligible work/components from the two schedulers; preserve batch-size/track settings, instrument suppressed/deferred work, and charge all actual updates. Version the ablation declaration that currently identifies this difference as batch_size.

**Acceptance gate:** Paired runs share initial conditions, random streams, external budgets, and component semantics; their declared difference is scheduler policy. Counters reconcile executed and suppressed work, no forbidden replay is introduced, and protocol completion cannot be faked with scheduler labels.

**Retained evidence:** Scheduling traces, one-difference manifest comparison, and paired resource/outcome report.

### [ ] BX-19 — Execute all eleven charter ablation pairs

**Milestone:** M5c. **Depends:** BX-18.\
**Legacy mapping:** BE-15; BE-09; BQ-12.\
**Reuse:** Extend orchestrate.py descriptors into the existing real runner.

**Objective:** Turn every mandatory treatment/control pair into executable, auditable experiments.

**Work and boundary:** Bind the eleven declared mechanisms to actual adapters/configurations. Freeze seeds, relevant environment versions, exogenous randomness, budgets, instrumentation, and analysis. Track B replay/dense/diagnostic controls retain their proper eligibility. The growth comparator keeps an outer safety cap.

**Acceptance gate:** Every pair launches both sides on at least one relevant diagnostic and produces verified bundles and a comparison. Runtime facts confirm the intended mechanism changed; undeclared differences or incompatible tracks invalidate the comparison. Mismatched action-dependent trajectories are not forced to be identical. Search/pilot data are labeled exploratory.

**Retained evidence:** Eleven runnable pair manifests, execution bundles, mismatch-negative tests, and coverage index.

### [ ] BX-20 — Construct C2–C5 evidence and statistics from immutable bundles

**Milestone:** M5c. **Depends:** BX-19.\
**Legacy mapping:** BV-05–BV-08; BK statistics; BE-16.\
**Reuse:** Reuse pure claim rules and statistics; add a verified evidence builder.

**Objective:** Make stronger claim inputs reconstructible rather than caller assertions.

**Work and boundary:** Resolve hashes, ancestry, matched comparisons, family/lifetime coverage, corrected statistics, resource costs, and energy tier from actual records. Freeze analysis before confirmatory data. Keep smoke verdicts honestly insufficient.

**Acceptance gate:** A fresh verifier derives the same metrics and verdicts from the same bundles. Forged booleans, missing seeds, comparator substitution, altered policies, removed failed runs, stale artifact links, or insufficient energy cannot manufacture PASS. Small exact statistical examples agree with an independent reference. Implementation closure does not require a positive agent claim; confirmatory campaigns occur at BX-39.

**Retained evidence:** Verified claim tables, adversarial provenance corpus, statistical checks, and preregistration records. M5c closes after BX-12–20.

### [ ] BX-21 — Implement the stable-dynamics changing-values family

**Milestone:** M5d. **Depends:** BX-20.\
**Legacy mapping:** BE-10.\
**Reuse:** Extend the existing revaluation scenario through the stepwise interface.

**Objective:** Isolate reward revaluation while transition dynamics remain stable.

**Work and boundary:** Provide a small world with an independently specified transition kernel and reward switches, plus a procedurally enlarged counterpart. Keep future switch information out of ordinary learner observations unless explicitly declared.

**Acceptance gate:** Kernel tests confirm unchanged transition dynamics across reward changes. A diagnostic verifies the expected revaluation intervention and timing independently of BRDC-1 performance. Both agents and relevant ablations run; reports separate reward changes from model/dynamics drift.

**Retained evidence:** Diagnostic oracle, enlarged manifest, paired runs, and causal-property tests.

### [ ] BX-22 — Implement the observation-aliasing family

**Milestone:** M5d. **Depends:** BX-21.\
**Legacy mapping:** BE-11.\
**Reuse:** Extend the aliasing family and observation mapper.

**Objective:** Create stationary hidden dynamics that require a richer state representation.

**Work and boundary:** Design observationally aliased states with different action consequences; add a diagnostic sufficient representation for evaluator-only causal analysis and an enlarged partial-observation case.

**Acceptance gate:** Observable collisions and differing conditional outcomes are verified from the true kernel. Stationarity is proved independently of sampled reward curves. Removing aliasing changes the specified diagnostic difficulty; privileged state never enters Track A input.

**Retained evidence:** Aliasing/sufficiency oracle, exposure tests, and raw versus diagnostic comparison bundles.

### [ ] BX-23 — Implement the late-arriving-factor family

**Milestone:** M5d. **Depends:** BX-22.\
**Legacy mapping:** BE-11.\
**Reuse:** Extend existing late-factor family and feature metrics.

**Objective:** Make a previously sufficient representation become inadequate after a new causal factor appears.

**Work and boundary:** Introduce a controlled factor with known activation and downstream effect; record the change only where permitted by manifest and track semantics. Add enlarged factor combinations without supplying the learner the correct feature.

**Acceptance gate:** Pre-activation and post-activation conditional predictions show the intended sufficiency change. A diagnostic intervention removing the factor restores the original relationship. Runs report feature acquisition cost, adaptation, and retained utility rather than only new-artifact counts.

**Retained evidence:** Causal factor tests, activation manifest, and feature-learning/ablation bundles.

### [ ] BX-24 — Implement the long-life-plasticity family

**Milestone:** M5d. **Depends:** BX-23.\
**Legacy mapping:** BE-12.\
**Reuse:** Extend plasticity family, lifetime metrics, and control baselines.

**Objective:** Distinguish declining ability to learn new tasks from forgetting previously learned tasks.

**Work and boundary:** Generate matched-difficulty tasks across learner age, with separate acquisition probes and return-to-old-task probes. Include enlarged lifetimes and fixed-capacity comparator conditions.

**Acceptance gate:** The world controls the documented difficulty variables across ages. Independent fixtures distinguish an agent that forgets from one that stops learning. Reports separately measure new-task learning, old-task retention, age, and cumulative budget use; no positive BRDC-1 trend is assumed.

**Retained evidence:** Task-difficulty contract, discrimination fixtures, age-stratified runs, and paired uncertainty.

### [ ] BX-25 — Implement the temporal-joints family

**Milestone:** M5d. **Depends:** BX-24.\
**Legacy mapping:** BE-13.\
**Reuse:** Extend temporal scenario, options, and planning measurements.

**Objective:** Provide actual temporally extended structure on which options can alter planning work.

**Work and boundary:** Use sustained action sequences and state transitions with analytically known useful temporal boundaries; add enlarged horizons and irrelevant candidate options.

**Acceptance gate:** Diagnostic tests establish the intended state/action dependencies and option duration. Primitive-only versus learned-option/model comparisons measure action quality and all construction/planning work. An option that merely repeats a label or changes reported duration cannot satisfy the gate.

**Retained evidence:** World oracle, executed option traces, work accounting, and relevant ablation bundles.

### [ ] BX-26 — Implement the distractor-abundance family

**Milestone:** M5d. **Depends:** BX-25.\
**Legacy mapping:** BE-14.\
**Reuse:** Extend distractor generator and feature-utility/curation metrics.

**Objective:** Separate predictive or salient features from features with downstream decision utility.

**Work and boundary:** Introduce controllable distractor quantities with known statistical structure but no effect on the diagnostic decision target. Preserve a small useful subset and add enlarged feature spaces.

**Acceptance gate:** Interventions on distractors leave diagnostic task consequences unchanged while prediction/salience can increase. Curation comparisons show measured retention choices and cost; a feature-count increase is not treated as utility. Noise seeds and task seeds are separately reproducible.

**Retained evidence:** Intervention oracle, scaling manifests, and utility/random/age-retention runs.

### [ ] BX-27 — Implement the resource-shock family

**Milestone:** M5d. **Depends:** BX-26.\
**Legacy mapping:** BE-14; BQ-07.\
**Reuse:** Extend policy revisions and existing resource-shock descriptors.

**Objective:** Apply real declared runtime resource changes and measure adaptation to them.

**Work and boundary:** Schedule supported compute/memory/deadline changes through the external governor, with versioned policy transitions and declared visibility. Define behavior when a lowered cap is already exceeded; preserve the outer safety ceiling.

**Acceptance gate:** Live workloads observe the actual controller/deadline changes at the intended boundary; records reconcile requested versus applied policy and overshoot. The no-shock control shares environmental dynamics. Unsupported changes are rejected or explicitly ineligible, never represented as applied.

**Retained evidence:** Policy timeline, controller readbacks, violation/overshoot traces, and shock/control bundles.

### [ ] BX-28 — Implement the model-mismatch family

**Milestone:** M5d. **Depends:** BX-27.\
**Legacy mapping:** BE-14.\
**Reuse:** Extend model-error metrics and mismatch scenario.

**Objective:** Make stale or biased temporal models measurably harm planning under controlled conditions.

**Work and boundary:** Change a known transition/duration mechanism while keeping reward changes separately controlled; provide diagnostic exact models only to the evaluator. Add enlarged nonlinear or longer-horizon mismatch within the declared reference scope.

**Acceptance gate:** Oracle tests distinguish model error from observation aliasing and pure reward revaluation. Planned and reactive controls expose the prescribed failure mode; the report localizes error by model age and horizon. Test fixtures cannot supply the desired claim verdict.

**Retained evidence:** Dynamics-intervention tests, model-error curves, and planning/reactive comparison bundles.

### [ ] BX-29 — Implement the recursive-reuse family

**Milestone:** M5d. **Depends:** BX-28.\
**Legacy mapping:** BE-13.\
**Reuse:** Extend hierarchical artifact lineage and recursive world descriptors.

**Objective:** Make higher-level tasks benefit from reusable lower-level learned structures.

**Work and boundary:** Create a diagnostic compositional task with known dependency structure and an enlarged variant. Record construction generations, actual consumer use, and the cost of maintaining reused artifacts.

**Acceptance gate:** Removing an identified lower-level dependency changes the predicted diagnostic capability. Lineage distinguishes shared ancestors from copied/renamed artifacts and avoids double-counted benefit. Matched no-reuse and frozen-representation runs produce independently verified ancestry/use records.

**Retained evidence:** Dependency oracle, multi-generation lineage, consumption events, and reuse ablations.

### [ ] BX-30 — Implement the noisy single-pass family and close family coverage

**Milestone:** M5d. **Depends:** BX-29.\
**Legacy mapping:** BE-12; BE-10–BE-16.\
**Reuse:** Extend noisy-stream generator, exposure checks, and family registry.

**Objective:** Test learning from controlled noisy experience with one authorized encounter per transition.

**Work and boundary:** Separate irreducible observation/reward noise from latent drift; support deterministic noise replay for the evaluator, never learner replay. Add enlarged stream conditions and map each of the ten families to its diagnostic/enlarged implementation and relevant ablations.

**Acceptance gate:** Known noise-free and noisy limits agree with independent diagnostic expectations. Runtime access/update records demonstrate one-pass semantics. Every family has a real mechanism test, an enlarged manifest, and an S run; five paired C seeds per family are scheduled under fixed manifests without silently weakening caps. Missing required runs keep M5d open.

**Retained evidence:** Noise/track tests; ten-family coverage matrix; S/C bundle index and actual resource usage.

### [ ] BX-31 — Record bounded candidate evolution and search costs

**Milestone:** M5e. **Depends:** BX-30.\
**Legacy mapping:** BC lineage; BK generation metrics; new between-run extension.\
**Reuse:** Extend artifact lineage, manifests, allocation accounting, and checkpoints.

**Objective:** Represent candidate generation as a reproducible, externally budgeted process.

**Work and boundary:** Record parent candidate, code/configuration/schema identities, checkpoint ancestry, allowed mutation operator, random seed, and search budget. Include failed/canceled candidates and evaluator costs. Keep mutation operators within the approved structures/pre-reviewed adapter boundary.

**Acceptance gate:** Identical parent/operator/seed produces the same candidate identity where deterministic. Invalid mutations cannot change the evaluator or escape the declared candidate space. Search budget exhaustion stops generation; every attempted candidate has a terminal record and reconciled costs. A metadata-only generation count cannot establish C5.

**Retained evidence:** Candidate lineage, mutation validation, resource ledger, and failed/canceled candidate records.

### [ ] BX-32 — Evaluate and promote candidates on held-out evidence

**Milestone:** M5e. **Depends:** BX-31.\
**Legacy mapping:** BE-16; BV-05–BV-08; new promotion seam.\
**Reuse:** Compose preregistration, real orchestration, verified claims, and statistics.

**Objective:** Select candidates using fixed evaluation rules that candidates cannot alter.

**Work and boundary:** Separate development/search and confirmatory seeds/environments; freeze evaluator/policy/metric versions before selection. Use staged promotion with declared regression constraints and uncertainty. Distinguish offline meta-level selection from within-run Track A adaptation; reveal only authorized scores to the search controller.

**Acceptance gate:** Changing held-out bundles, selecting after looking at confirmatory data, changing metrics/budgets mid-comparison, or reusing hidden seeds incorrectly invalidates promotion. Candidates with a known regression or incomplete evidence are rejected/indeterminate, and a qualifying diagnostic candidate is promoted reproducibly. A candidate process cannot read the evaluator-only artifacts through its authorized channels.

**Retained evidence:** Frozen selection manifests, provenance/selection-negative tests, and actual promotion decisions.

### [ ] BX-33 — Recover and roll back across candidate generations

**Milestone:** M5e. **Depends:** BX-32.\
**Legacy mapping:** BR recovery; BC migration; BV-12.\
**Reuse:** Extend checkpoints, version migration, and run supervisor.

**Objective:** Demonstrate reversible, auditable evolution without losing the accepted parent state.

**Work and boundary:** Preserve immutable accepted parents; validate checkpoint compatibility before activation; detect failed/regressed candidates and return to a known compatible parent. Exercise at least three candidate generations as a mechanism test, not a threshold for open-ended intelligence.

**Acceptance gate:** Faults before activation, during child execution, and after result creation never corrupt the accepted parent or erase evidence. Rollback produces the expected parent checkpoint hash and valid new-run provenance. Incompatible states fail closed. Agent state carried between generations is explicitly classified so Track A and fresh-agent claims are not conflated.

**Retained evidence:** Three-generation run chain, injected-failure records, rollback hashes, and checkpoint compatibility report. M5e closes after BX-31–33.

### [ ] BX-34 — Implement Windows process-tree measurement and controls

**Milestone:** M5f. **Depends:** BX-33.\
**Legacy mapping:** BM-05–BM-06; BQ-07; BR-03.\
**Reuse:** Extend Windows platform backend and common transport supervisor.

**Objective:** Replace unsupported Job Object scaffolding with verified Windows behavior where APIs permit.

**Work and boundary:** Use an established safe binding or narrowly reviewed native helper for job attachment, accounting, supported limits, and kill-on-close semantics. Identify nested-job/breakaway behavior and unsupported counters. Keep library/FFI exceptions explicit and local.

**Acceptance gate:** Live Windows tests observe real child/grandchild membership, CPU/memory accounting, intended limit crossings, pipe/cancellation behavior, and cleanup. Race-window and nested-job fixtures exercise launch/attach ordering. A permission failure remains a capability failure, not a successful enforcement record.

**Retained evidence:** Windows integration traces, controller/accounting reconciliation, and capability matrix; physical qualification closes at BX-36.

### [ ] BX-35 — Implement macOS measurement and bounded termination

**Milestone:** M5f. **Depends:** BX-34.\
**Legacy mapping:** BM-07–BM-08; BQ-07; BR-03.\
**Reuse:** Extend macOS backend and transport process-group support.

**Objective:** Deliver honest supported controls and measured monitor/terminate behavior on macOS.

**Work and boundary:** Implement process-tree measurement, supported resource controls, and cancellation. Where a hard kernel limit is unavailable, explicitly expose monitor/terminate semantics and measure overshoot; never advertise equivalence to a hard control.

**Acceptance gate:** Live macOS workloads validate supported counters, descendant coverage, timeout/termination, inherited-pipe handling, and cleanup. Tests measure overshoot distributions and reject manifests requiring unavailable hard guarantees. Sensor/privilege limitations remain explicit.

**Retained evidence:** macOS integration and overshoot records; capability matrix; physical Apple-silicon qualification at BX-36.

### [ ] BX-36 — Qualify required physical backends and available energy tiers

**Milestone:** M5f. **Depends:** BX-35.\
**Legacy mapping:** BM-11–BM-14; BV-09.\
**Reuse:** Reuse calibration workloads, NVML/energy collectors, and equivalence reports.

**Objective:** Establish physical-host evidence for every required platform without pretending their capabilities are identical.

**Work and boundary:** Run the complete governed conformance workload on Windows 11 x86_64, Linux x86_64 cgroup v2, and Apple-silicon macOS. Calibrate available counters and optional NVIDIA/Apple energy paths; apply the existing E0–E3 rules. Record absent equipment honestly.

**Acceptance gate:** All three required physical hosts have attested controller/measurement/cleanup records and semantic protocol/bundle equivalence. Numeric performance is compared only within declared comparable conditions. Missing optional energy hardware can remain E0/E1, blocking only claims requiring more; a missing required physical OS keeps this gate open.

**Retained evidence:** Physical-host attestations, calibration uncertainty, capability/comparability matrix, and conformance bundles.

### [ ] BX-37 — Close instrumentation overhead and observer scaling

**Milestone:** M5f. **Depends:** BX-36.\
**Legacy mapping:** BK-14; BM-04; BV-10; D-11.\
**Reuse:** Extend existing overhead runner, metrics, and report paths.

**Objective:** Measure and meet the canonical overhead ceiling on representative real experiments.

**Work and boundary:** Run paired no/minimal/full instrumentation experiments on the qualified hosts with fixed work and semantics. Attribute hot-path costs, remove unnecessary report/table work from the run path, and bound queues/indexes. Safety controls remain active in all arms; incomplete no-instrumentation arms are not claim bundles.

**Acceptance gate:** The paired confidence bounds meet D-11 throughput and p95-latency thresholds with zero required-event loss in the evidence-producing configuration. Include primitive and discovery workloads with stated event rates/live-state sizes. Independent metrics agree within declared numeric tolerances; a failure remains open, not fixed by suppressing required events.

**Retained evidence:** Paired overhead data, confidence intervals, resource profiles, scaling results, and any focused extraction ADR.

### [ ] BX-38 — Validate supported toolchains, dependencies, and clean offline restore

**Milestone:** M5f. **Depends:** BX-37.\
**Legacy mapping:** BG-05; BV-13; D-02; D-08.\
**Reuse:** Extend lockfiles, CI, SBOM and restore scripts.

**Objective:** Keep the codebase current where justified while preserving reproducible past experiments.

**Work and boundary:** Audit current official supported releases/security advisories at execution time; update one compatible dependency/toolchain set with a recorded rationale. Test declared Python 3.12–3.14 support or approve a narrowed range. Keep a pinned experiment lane and a current-stable Rust compatibility lane. Build a real offline dependency closure.

**Acceptance gate:** Declared Python versions pass applicable tests/types/lint; native targets pass the approved matrix. A clean isolated restore with network disabled installs/builds/tests from the retained dependency archive, not an existing cache. SBOMs match lockfiles; known material advisories are resolved or explicitly recorded under policy. No automatic upgrade of archived manifests.

**Retained evidence:** Version/support decision, before/after compatibility results, SBOM hashes, clean offline restore logs, and cache-size ledger.

### [ ] BX-39 — Execute preregistered confirmatory and long-duration campaigns

**Milestone:** M5f. **Depends:** BX-38.\
**Legacy mapping:** BE-16; BV-05–BV-06; BV-14.\
**Reuse:** Compose real runner, freeze manifests, physical hosts, and claim evidence builder.

**Objective:** Replace short duration probes and not-run placeholders with actual research and L-profile evidence.

**Work and boundary:** Freeze the exact A/L run matrix, seed counts, selected relevant families/ablations, lifetime/generation metrics, host assignments, disk reservations, cost estimates, and interruption policy before launch. Use only authorized available hosts and approved budgets. Run campaigns, preserve failures, and independently derive verdicts.

**Acceptance gate:** Confirmatory outcomes meet the canonical sample/analysis prerequisites for whatever claim is attempted; outcomes may fail scientifically. L evidence shows at least 72 h AND 10M steps for each of three paired seeds per required physical OS, actual process/resource histories, and valid interruption classification. No busy-loop probe, elapsed-idle time, or stitched unclassified restart can substitute. Missing hosts or run budget keeps this prompt BLOCKED/NOT-RUN.

**Retained evidence:** Signed/frozen campaign manifest, per-run bundles/attestations, seed/failure ledger, resource totals, and independently reconstructed claims.

### [ ] BX-40 — Deliver reproducible handoff and evidence-based completion audit

**Milestone:** M5f. **Depends:** BX-39.\
**Legacy mapping:** BV-15–BV-16; all claim/source ledgers.\
**Reuse:** Extend operator docs, RC packaging, claim matrix, and existing logs.

**Objective:** Make the completed instrument usable and accurately describable by a fresh operator.

**Work and boundary:** Document install/run/analyze, agent/environment extension, evolution/rollback, limitations, and recovery using actual final commands. Produce local RC artifacts and an evidence index. Reconcile all BX gates and older gaps, archive draft authority correctly, and inventory retained worktrees/caches without unauthorized removal.

**Acceptance gate:** A fresh operator/environment reproduces a small end-to-end run and independently verifies its bundle from documentation alone. Final packages/hashes/SBOM/restore checks pass. All mandatory BX gates have commit/source/CI or physical evidence as appropriate; unresolved required gates prevent M5 completion. No package publication/tag or positive C4/C5 announcement follows automatically.

**Retained evidence:** Operator reproduction log, final claim/evidence matrix, RC hashes, complete DEVLOG/verification index, and storage/worktree closeout.

## 5. Reuse ledger

Before adding a crate, framework, schema, store, or registry, the owning BX prompt must inspect the named existing implementation and record why extension is insufficient. New code is justified by an executable capability gap, not by a preference for a new abstraction.

| Classification | Existing asset or seam | Planned use | BX ownership |
|---|---|---|---|
| Reuse unchanged where valid | Charter, D-01–D-21, track definitions, statistical/energy rules, governance logs | Preserve scientific and execution authority | 01; all |
| Extend | ChildTransport and adapter protocol machine | Bounded I/O, cancellation, process ownership, compatibility | 02, 09, 34–35 |
| Extend | Linux/Windows/macOS platform modules and governor bridge | Actual controls, readbacks, capability qualification | 03, 27, 34–36 |
| Extend at existing seam | ScenarioSpec, reference environment protocol | Stepwise causal interaction and family-specific mechanisms | 04, 21–30 |
| Compose | Runtime, governor, ingestor, bundles, control learner, report | Real operator run path; add a thin CLI target only if necessary | 05–06 |
| Extract then extend | Existing full-trace lineage legality validator | Incremental state with replay retained as oracle | 07 |
| Extend | Event segments, blobs, SQLite index, recovery | Bounded persistence, checkpoints, historical queries | 08 |
| Reuse standard | Protobuf compatibility rules, SQLite, Arrow/Parquet | Stable interchange and existing storage tools | 08–09 |
| New adapter implementation at existing seam | Public baseline algorithms; optional Gymnasium API | Independent agent/environment demonstrations | 10–11 |
| Extend | BRDC-1 stages and artifact lifecycle types | Online discovery, learned models, planning, credit, curation | 12–17 |
| Extend | Dense/event schedulers and ablation descriptors | Actual measured treatment execution | 18–19 |
| Compose and extend | Pure claim rules, metrics, stats, hash verifier | Trusted bundle-derived evidence builder | 06, 20 |
| New logic using existing contracts | Causal diagnostic mechanisms | Named family behavior beyond parameter/label variation | 21–30 |
| New bounded functionality at existing seams | Candidate lineage, promotion, rollback | Controlled between-run evolution and search accounting | 31–33 |
| Extend and extract only if measured | Existing reports and metric derivation | Remove observed action-path overhead without a second stack | 37 |
| Extend | Lockfiles, CI, SBOM, offline/RC tooling | Supported modern toolchains, clean restoration, operator delivery | 38–40 |

## 6. Review-finding traceability

| Architecture-review finding | Response | Required evidence before closure |
|---|---|---|
| Experiment descriptors without full execution | BX-04–06, 19; 21–30 | Real child interaction, real controls and all pair/family run indexes |
| Claim input assertions instead of derived facts | BX-06, 15–16, 20 | Independent bundle reconstruction; exact counterfactuals; invalid provenance fails |
| Full-history lineage cloning/revalidation | BX-07–08 | Differential correctness, prefix-independent event processing, bounded memory/recovery |
| Unbounded I/O/join and descendant survival paths | BX-02, 34–35 | Live blocked-pipe/grandchild fixtures with deadline and survivor evidence |
| Native control scaffolding and unqualified hosts | BX-03, 34–36, 39 | Controller effects and real physical-host/long-run records |
| BRDC-1 supplied artifacts/deltas mistaken for autonomous learning | BX-12–17 | Experience-generated proposals, learned behavior, measured counterfactual utility |
| Flexibility asserted but not demonstrated | BX-09–11 | Compatible historical adapters and materially different agent/environment integrations |
| Self-evolution ambition without evaluation boundaries | BX-31–33 | Budgeted ancestry, held-out promotion, stable evaluator, verified rollback |
| Toolchain/support and acceptance proof gaps | BX-37–40 | Declared runtime matrix, current-release audit, real offline restore, overhead and A/L evidence |

These are follow-up obligations, not a retrospective claim that all earlier tests were worthless. Existing tests remain regression evidence for the behaviors they actually establish.

## 7. Risk, dependency, and blocker register

| ID | Risk or prerequisite | Owner / first resolving prompt | Rule |
|---|---|---|---|
| R5-01 | Local checkout differs from live main; concurrent work may appear | Implementation owner / BX-01 | Preserve exact work before advancing; do not overwrite or repeat already-fixed changes. |
| R5-02 | Ubuntu location/type or delegated cgroup access not established for new work | Operator + implementation owner / BX-01–03 | Identify distro/host/mount/capabilities; do not treat WSL as physical or change global host controls implicitly. |
| R5-03 | Required physical Windows/Linux/macOS availability | Operator / BX-01, BX-36 | Plan can progress only within approved dependencies; missing physical evidence remains open. |
| R5-04 | Native binding safety policy conflicts | Runtime owner / BX-02, 34–35 | Prefer established safe wrappers; review any local helper/FFI decision explicitly; no blanket unsafe-policy relaxation. |
| R5-05 | False scientific confidence from labels, tiny fixtures, or proxy utility | Scientific owner / BX-12–30 | Causal oracles, exact diagnostics, independent derivation, and preregistered held-out evidence. |
| R5-06 | Candidate search leaks evaluation data or hides cost | Scientific + runtime owners / BX-31–33 | Separate evaluation access, track/meta-level semantics, and failed-candidate cost ledger. |
| R5-07 | Long-run disk/compute exhaustion and workstation interference | Operator / BX-01, 39 | Freeze resource reservation, total campaign count, retained-output size, and host scheduling before launch. |
| R5-08 | A required energy tier is unavailable | Measurement owner / BX-36 | Do not block honest E0 instrument operation; block only the unsupported energy/C5 claim. |
| R5-09 | New work invalidates archived protocol or experiment reproduction | Contracts owner / BX-09, 38 | Versioned compatibility, explicit migrations, immutable old manifests/pins. |
| R5-10 | Approval confusion after moving to a project session | Task owner / handoff | DRAFT stays DRAFT; the new session verifies any subsequent user approval and exact authorized cut. |

For scale planning, three paired L seeds on each of three OS targets means **18 arm-runs** if each pair contains two arms. At 72 hours per arm this is at least **1,296 host-hours**, before retries, calibration, confirmatory A runs, or additional ablations. This is arithmetic for the proposed campaign, not a booking, cost quote, promise of elapsed completion time, or authorization to run hosts concurrently. BX-39 must freeze the exact interpretation and allocation against D-16 before launch. Any smaller accepted scope must be recorded as a revised gate rather than called full L acceptance.

No privileged installation, additional hardware, paid cloud capacity, or unattended campaign is assumed available merely because it appears in the roster. Use previously granted specific authorization when present; ask only for genuinely missing host/resource authority when the concrete run manifest is ready.

## 8. Worktree and storage discipline

The canonical checkout is the default single active lane. Do not create a new worktree for this sequential roster. If concurrent work later makes isolation necessary, first inventory existing trees, confirm none is suitable, check disk capacity, and record owner, purpose, branch, and retirement condition. New branches, if needed, use the codex/ prefix.

Read-only inventory on 2026-09-05:

| Location / purpose | Dirty state and unique commits | Generated state measured | Retirement status |
|---|---|---|---|
| Canonical Reinforcement Learning Project checkout / active BONSAI | Clean; main nine commits behind verified live main | At least 8.15 GiB combined target and .venv; three target subdirectories were unreadable, so this is a lower bound | Retain as canonical; caches subject to explicit, preservation-aware cleanup only |
| C:\tmp\bonsai-m0-sts / historical M0 worktree, branch codex/m0-governed-foundation, HEAD eaa0e52ec5a6dc78ab1a360f2a11c2201c7a5e9d | Clean; zero commits unique relative to cached origin/main 2cdd26d; no claim about a separately published branch tip | Approximately 1.36 GiB combined target and .venv | Existing cleanup debt; current active need and exact preservation must be rechecked before any authorized removal |

No tree, branch, cache, or user artifact was removed during drafting. At each milestone closeout, refresh this inventory, including staged/unstaged/untracked work, unpublished commits, generated size, and retirement blocker. Never use force removal to bypass an uncertain dirty state. Removal requires explicit user authorization even if a branch appears merged.

Raw telemetry is not committed by default. Keep small fixtures/manifests, redacted evidence indexes, hashes, and exact artifact locations in the repository. Retain large bundles in the operator-approved project evidence location under D-16 caps. Avoid duplicated dependency trees and high-volume files in chat outputs or Git history. The new session should use the canonical checkout and existing suitable environments.

## 9. Parked scope and explicit exclusions

P-01–P-09 remain parked: E3 laboratory probes; cluster/cloud-required execution; mobile/robotic/real-world control; new AMD/Intel discrete collectors; hostile-code sandboxing; a mandated neural architecture or general-intelligence proof; unpublished Oak Lab reproduction or representation; external submissions/SaaS/telemetry upload; and human labels in Track A.

Additional boundaries for this slice:

- No UI redesign, new web service, distributed job platform, general plugin marketplace, or replacement of the repository's governance format.
- No arbitrary source-generating agent that rewrites BONSAI, changes its evaluator, deploys itself, or acquires credentials. A separately scoped evaluator/containment design would be needed before broadening this boundary.
- No silent reuse of held-out evaluation as learner replay, nor treating meta-level search as unassisted Track A learning.
- No source-independent benchmark trophy, positive C4/C5 requirement, tag, public release, package upload, or marketing claim.

## 10. Execution record and closeout format

Use the existing DEVLOG and BONSAI-VERIFICATION-LOG; do not create a rival execution ledger. Each BX record includes:

1. Prompt ID, approved scope/authority, dependency baseline, objective, and reuse classification.
2. Files changed, new/superseded ADRs or schemas, exact tested source identity and patch hash where relevant.
3. Exact commands, timestamps, exit codes, full retained failure output, host/runner identity, artifact locations and hashes.
4. Separate implementation verdict, scientific result, local/hosted/physical proof, and not-run/blocked conditions.
5. Focused commit identity, authorized remote/main SHA and required CI result when published, using the established self-hash closeout convention.
6. Changed risks/parks, next eligible prompt, and worktree/storage closeout.

A prompt status is [ ] not started, [~] executing, [x] gated and recorded, [!] blocked, or [-] superseded by a named approved addendum. Scope changes and deferred gates must preserve historical text and evidence.

## 11. Completion criteria

**Draft ready for review:** governance and scope explicit; parent/history preserved; all 40 prompts have one focused objective, dependencies, reuse mapping and observable acceptance; milestone cuts and resources are defined; source references and handoff exist. This is the only completion being claimed by the current drafting task.

**M5 implementation complete, only after authorized execution:**

1. BX-01–BX-40 pass their mandatory implementation/evidence gates and are recorded at exact source identities.
2. The supervisor executes real agent/environment interactions and enforces the capabilities it advertises.
3. Lineage/persistence remain bounded and recoverable, with compatible archived evidence and independent verification.
4. Distinct agents/environments integrate through stable contracts without instrument-specific algorithm branches.
5. BRDC-1 construction, planning, backward credit and curation execute from experience; all ten causal families and eleven ablation pairs are runnable and verified.
6. Candidate evolution is externally budgeted, versioned, held-out evaluated, and reversible with honest meta-level/Track A semantics.
7. Required physical targets, overhead, supported-runtime matrix, offline restore, and actual A/L acceptance are evidenced. Mandatory not-run rows keep full completion open.
8. A fresh operator reproduces a run and verifies its results; final RC/source/evidence indexes and storage inventory are complete.
9. Scientific claims remain exactly those supported by the bundles. Instrument completion does not require an agent to pass C4/C5; E0/E1 results cannot claim unsupported total-energy positivity.

A user-approved narrower milestone can close on its own criteria. It must not be renamed “BONSAI complete” while the remaining cuts are open.

## 12. Approval record

| Field | Current value |
|---|---|
| Draft version | 0.5-draft.1, 2026-09-05 |
| Plan approval | **PENDING** |
| Approved changes/defaults | **NONE YET** |
| Execution authorization | **NONE FOR BX-01–BX-40** |
| First recommended cut | M5a / BX-01–BX-06 |
| First eligible prompt after authorization | BX-01 |
| Source commits/pushes during this draft | None |
| Host experiments during this draft | None |
| Subsequent authorization record | Append the user's exact approval/STS wording, date, and scope; never infer it from moving the task into Projects |

## 13. Evidence and reference index

References are pinned to the verified remote review baseline unless otherwise specified. Current execution must recheck them at BX-01.

- [Canonical PSPR v0.1 and D-01–D-21](https://github.com/USS-Parks/BONSAI-Research-Labs/blob/2f35355c6d12d019eb8625cb3bd38728d90ee029/BONSAI%20Research%20Charter/BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md).
- [Research charter: families, ablations and claims](https://github.com/USS-Parks/BONSAI-Research-Labs/blob/2f35355c6d12d019eb8625cb3bd38728d90ee029/BONSAI%20Research%20Charter/BONSAI-RESEARCH-CHARTER.md).
- [M3 approved slice and physical-capability exceptions](https://github.com/USS-Parks/BONSAI-Research-Labs/blob/2f35355c6d12d019eb8625cb3bd38728d90ee029/BONSAI%20Research%20Charter/BONSAI-PSPR-v0.3-M3-CROSS-PLATFORM-GOVERNED-SCIENCE.md).
- [M4 approved slice, OD-01 and publication boundary](https://github.com/USS-Parks/BONSAI-Research-Labs/blob/2f35355c6d12d019eb8625cb3bd38728d90ee029/BONSAI%20Research%20Charter/BONSAI-PSPR-v0.4-M4-ACCEPTANCE-AND-RELEASE.md).
- [DEVLOG execution history](https://github.com/USS-Parks/BONSAI-Research-Labs/blob/2f35355c6d12d019eb8625cb3bd38728d90ee029/docs/sessions/BONSAI-DEVLOG.md).
- [Continuing source-publication addendum](https://github.com/USS-Parks/BONSAI-Research-Labs/blob/2f35355c6d12d019eb8625cb3bd38728d90ee029/docs/governance/addenda/2026-07-19-later-pspr-source-publication.md).
- [Parked-scope ledger](https://github.com/USS-Parks/BONSAI-Research-Labs/blob/2f35355c6d12d019eb8625cb3bd38728d90ee029/docs/governance/PARKED-SCOPE-LEDGER.md).
- Companion review: BONSAI-Architecture-Review-2026-09-05.md in this draft's output folder; contains pinned source links and review verification limits.
- Companion transfer note: BONSAI-Project-Session-Handoff-2026-09-05.md in the same output folder.

---

End of PSPR v0.5-draft.1. **Drafting complete; review and fresh execution authorization pending.**

## 14. Approval and full-roster STS authorization — 2026-09-05

The user approved this plan and authorized execution with the exact instruction:

> Approved for STS now. Authorization to run this until verified complete, committing and merging with main after each prompt is finished fully, and all 40 prompts are successfully executed.

Scope: all 40 prompts BX-01–BX-40 in dependency order, one fully gated prompt per focused commit, merged/published to Labs main after each prompt. Existing gate, physical-host, resource, and separately controlled action boundaries remain in force. Section 12 and the end-of-draft statement above preserve the pre-approval record; this section records the subsequent authority.

BX-01 is executing at source baseline `2f35355c6d12d019eb8625cb3bd38728d90ee029`. Imported documentation changes are preserved and included in this prompt. BX-02–BX-40 remain unstarted. No gate has yet been closed under v0.5.

### BX-01 closeout / BX-02 start — 2026-09-05

BX-01 passed local gates and hosted run [33998986034](https://github.com/USS-Parks/BONSAI-Research-Labs/actions/runs/33998986034) on corrected main commit `51637ee2dfb3b21673bfeddecb8533a8dd6e29d5`. Initial commit `25d2c8e40a94fb2ea14208db177413c51b182681` required one documented evidence-byte correction. All four hosted OS/architecture jobs and M1 semantic equivalence passed; committed output hashes match machine records. BX-02 is now executing; BX-03–BX-40 remain unstarted.

### BX-02 closeout / BX-03 start — 2026-09-05

BX-02 closed at main `9a2457afd873bac167ffd61602b64f624f7ea089`, hosted [run 33999964570](https://github.com/USS-Parks/BONSAI-Research-Labs/actions/runs/33999964570) fully successful on all four platforms and M1 semantic equivalence. Eight committed output hashes and nine implementation hashes matched their manifests. BX-03 is executing; physical acceptance remains open.
