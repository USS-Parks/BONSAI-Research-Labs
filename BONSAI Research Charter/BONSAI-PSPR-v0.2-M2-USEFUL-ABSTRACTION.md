# BONSAI PSPR v0.2 — M2 Useful-Abstraction Science Slice

**Benchmark for Online, Nonstationary, Single-pass Agent Intelligence**

Version: 0.2 (APPROVED)  
Date: 2026-08-23  
Status: **APPROVED v0.2** 2026-08-23 (PT) by Basho Parks — OD-01–OD-03 settled; **NOT AUTHORIZED FOR EXECUTION** (awaits `run M2-science STS` / `run it STS`)  
Author: drafted for Basho Parks; approved by Basho Parks 2026-08-23 (PT)  
Authoritative local root: `C:\Users\17076\Documents\Reinforcement Learning Project`  
Remote: `USS-Parks/BONSAI-Research-Labs` (`main` at BQ-05 `1aa0751` or later)  
Parent PSPR: [BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md](./BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md) (v0.1, approved 2026-07-18)

> **Plan approval is not authorization to execute.**  
> This PSPR v0.2 was **approved** by Basho Parks on 2026-08-23 (PT). OD-01–OD-03 are settled (accept defaults).  
> No prompt in this roster may begin until Basho Parks says `run it STS` or `run M2-science STS`, or explicitly authorizes named milestone cuts or prompt IDs from this document.  
> Prior `Continue to STS` authority applied to the v0.1 roster only (**OD-02**); v0.2 requires a fresh, explicit STS phrase.

---

## 0. Purpose and boundary

### 0.1 User-chosen goal

Finish the **useful-abstraction / M2 science slice**:

1. **Lineage utility** — feature → subproblem → option → model → planning artifacts are traceable with backward utility credit.
2. **BRDC-1** — BONSAI Reference Discovery Cycle v1 runs end to end under external budgets without OaK-reproduction claims.
3. **C3 claims** — machine adjudication yields **pass / fail / indeterminate** for useful abstractions under controlled diagnostic ablations and statistical rules.

This PSPR does **not** claim OaK reproduction, instrument completion, C4/C5 pass, physical-host energy closure, long-duration L acceptance, or public release packaging.

### 0.2 Relation to v0.1

| Item | Disposition |
|---|---|
| Completed M0 / M1 / M2-runtime prompts | Remain authoritative history. Do **not** re-implement. |
| v0.1 full roster (BG→BV-16) | Remains the historical approved plan and charter-to-prompt map. |
| Remaining M2 science work | **Scoped and ordered here** as the approved (not yet STS-authorized) execution roster for the science slice. |
| M3 / M4 and non-science M2 leftovers | **Parked** for later PSPR addenda (see §8). |

**Already complete (do not re-implement):**

- M0: BG-01–BG-10
- Contracts: BC-01–BC-12
- M1: BR-01–BR-06, BM-01–BM-04, BQ-01–BQ-04, BK-01–BK-03, BE-01–BE-03, BV-01–BV-03
- M2 runtime so far: BR-07–BR-10, BQ-05 (HEAD at drafting: `1aa0751`)

**v0.1 M2 cut (for reference):** BR-07–BR-10, BQ-05–BQ-06, BK-04–BK-13, BE-04–BE-09, BV-04–BV-05.

**This v0.2 remaining set:** BQ-06, BK-04–BK-13, BE-04–BE-09, BV-04–BV-05 (**19 prompts**).

### 0.3 Source-of-truth hierarchy

Same as v0.1 / `docs/governance/SOURCE-OF-TRUTH.md`, with this addition:

4a. **After approval**, this PSPR v0.2 is authoritative for **M2-science-slice** implementation order, prompt scope, gates, and parked deferrals. It does not rewrite completed evidence or weaken the charter.

When this document conflicts with v0.1 on *remaining* M2 science scope, **this approved PSPR v0.2 wins** for M2-science-slice order, scope, gates, and parks. Execution still requires a fresh STS phrase (**OD-02**).

### 0.4 Prompt ID policy

Stable IDs are **kept identical** to v0.1 (`BQ-06`, `BK-04`…`BK-13`, `BE-04`…`BE-09`, `BV-04`, `BV-05`) to avoid drift. Optional alias prefix `M2S-` may be used in session notes as `M2S:BQ-06`, etc.; the canonical ID remains the original.

Checkbox semantics unchanged: `[ ]` `[~]` `[x]` `[!]` `[-]`.

### 0.5 Universal verification gate

Same as v0.1 §0.6. After scaffold commands exist, every implementation prompt must pass the applicable subset of:

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo test --workspace --all-features`
4. `uv run ruff check .`, `uv run pyright`, and `uv run pytest` for Python/reference-science changes
5. `cargo xtask schema-check` for schema/manifest/event/bundle/API changes
6. `cargo xtask evidence-check` for evidence-producing changes
7. Platform-specific integration tests on any platform actually claimed
8. No new ignored/quarantined/expected-failure test without risk-register entry and waiver
9. No mock-only closure for resource enforcement, process isolation, replay isolation, energy/counter collection, tamper detection, or platform behavior
10. Prompt-local verification record: exact command, start/end, exit code, source revision, platform fingerprint, artifact hashes

One prompt → one focused commit (or justified inseparable bundle in the DEVLOG). A failed or indeterminate scientific result is evidence, not a reason to weaken a gate.

---

## 1. Settled carry-forward decisions

**D-01 through D-21 from v0.1 still apply** unless noted.

| ID | Carry-forward note for this slice |
|---|---|
| D-01–D-08, D-10–D-13, D-21 | Unchanged. |
| D-09 | License remains `MIT OR Apache-2.0`. **Visibility override already approved:** public repository under the 2026-07-18 public-repository addendum. Further publication/release actions still need separate authority. |
| D-14 | Energy tiers unchanged. This slice does **not** pursue C5 energy positivity; E0/E1 availability notes remain honest. |
| D-15 | C2/C3 candidate claims still require ≥20 paired seeds, preregistered outcomes, paired 95% bootstrap intervals, effect sizes, Holm correction. Smoke/conformance counts unchanged for fixture gates. |
| D-16 | Smoke/Conformance profiles are in-scope for diagnostic runs. Acceptance A and Long L are **parked** for this slice. |
| D-17 | **BRDC-1** remains the named reference cycle: tabular/linear public ingredients; not an Oak Lab implementation. |
| D-18 | Prefer BONSAI-owned deterministic diagnostic worlds for this slice. Full ten-family enlargement is parked. |
| D-19 | Consequential-backup definition unchanged (paired counterfactual). |
| D-20 | Utility hierarchy unchanged; **proxy utility alone cannot establish C3**. |

No new D-IDs are introduced in this document. OD-01–OD-03 were settled by Basho Parks accepting defaults on 2026-08-23 (PT); see handoff §4. This document is **approved** as the M2-science plan; execution still requires a fresh STS phrase.

---

## 2. Milestone cuts for this slice only

Milestones below are independently approvable **within this science slice**. They do not authorize M3/M4 or publication.

| Milestone | Outcome | Prompt cut | Milestone gate |
|---|---|---|---|
| **M2a — Lineage utility** | Continual, feature, subproblem, option, model, planning, utility, cycle-health, failure, and statistical metrics exist; agent storage/replay guard protects Track A persistence. | BQ-06, BK-04–BK-13 | Synthetic/controlled fixtures yield exact or labeled-tier utility/lineage metrics; proxy sign/confidence failures → indeterminate; replay-buffer fixtures classified; no OaK claim. |
| **M2b — BRDC-1 reference cycle** | BRDC-1 specified and implemented through curation with exact diagnostic ablations via comparators. | BE-04–BE-09 | Traceability table present; feature→subproblem→option→model→planning→backward credit→curation reconstructs from bundles; each comparator is one intentional difference; Track A isolation holds. |
| **M2c — C3 adjudication** | Machine C0–C3 rules produce pass/fail/indeterminate; C3 requires controlled ablation + statistics, never proxy-only. | BV-04–BV-05 | Synthetic and reference diagnostic corpora cover every C2/C3 prerequisite verdict; no comparator-track leakage; no OaK reproduction claim. |

**Slice-level completion gate (supersedes the informal reading of v0.1 M2 for this approved slice):**

1. All 19 prompts are `[x]` with DEVLOG + verification records and focused commits.
2. Traceable feature → subproblem → option → model → planning artifacts with backward utility credit appear in at least one BRDC-1 diagnostic bundle.
3. Exact diagnostic ablations (via BE-09 comparators + BK-10 leave-one-out / matched attribution) are runnable and evidenced.
4. C3 is adjudicated by machine rules as pass, fail, or indeterminate — never silently pass on missing evidence.
5. Reports/bundles remain honest: no instrument-completion, C4/C5, or OaK-reproduction claim.
6. **Diagnostic-family coverage:** at least three distinct BONSAI diagnostic worlds (under BE-01 protocol) exercise distinct charter-family *properties* needed for C3 adjudication. Full BE-10–BE-14 family implementations remain parked (**OD-01 SETTLED 2026-08-23 PT:** accept default; no BE-10… pull-in).

---

## 3. Dependency order (minimal)

```text
Already done ──► BQ-06 ─────────────────────────────────────────┐
Already done ──► BK-04                                          │
Already done ──► BK-05 → BK-06 → BK-07 → BK-08 → BK-09 → BK-10 │
                                              └→ BK-11 → BK-12  │
Already done ──► BK-13 (may proceed after BK-01 anytime)        │
BK-05..BK-11 ──► BE-04 → BE-05 → BE-06 → BE-07 → BE-08 → BE-09 │
Already done ──► BV-04 ─────────────────────────────────────────┤
BV-04 + BK-04..BK-10 + BK-13 + BE-09 ──► BV-05 ◄───────────────┘
```

Recommended serial STS order (respects dependencies; permits noted parallelism only when explicitly authorized):

1. BQ-06  
2. BK-04  
3. BK-05  
4. BK-06  
5. BK-07  
6. BK-08  
7. BK-09  
8. BK-10  
9. BK-11  
10. BK-12  
11. BK-13  
12. BE-04  
13. BE-05  
14. BE-06  
15. BE-07  
16. BE-08  
17. BE-09  
18. BV-04 *(may be pulled earlier after M1 deps; listed late only for milestone packaging)*  
19. BV-05  

If Basho authorizes parallel worktrees: `{BQ-06 ‖ BK-04 ‖ BV-04 ‖ BK-13}` may start together; `{BK-05…}` remains serial among itself; BE\* remains serial after BK-11; BV-05 is last.

---

## 4. Sequential prompt roster

Objectives, files, excludes, and gates are carried forward from v0.1 with slice-local notes. Status at drafting: all `[ ]`.

### 4.1 M2a — Lineage utility

- [x] **BQ-06 — Agent storage and replay guard.** **Depends:** BR-06, BQ-04. **Files:** storage broker/policy. **Objective:** meter authorized agent persistence, deny observer paths, enforce bytes/files growth, and classify transition-like retention. **Excludes:** banning all learned parameters or legitimate state. **Gate:** replay-buffer fixtures are detected/classified; model parameters and bounded algorithm state remain allowed; path/symlink traversal fails. **Slice note:** required so BRDC-1 Track A runs cannot silently retain transition replay.

- [x] **BK-04 — Continual-learning metrics.** **Depends:** BK-02. **Files:** continual metric modules. **Objective:** retention, adaptation, plasticity loss separated from forgetting, transfer/interference, relearning, divergence, and age curves. **Excludes:** conflating failure to retain with inability to learn new structure. **Gate:** synthetic trajectories independently manipulate forgetting and plasticity and receive distinct results. **Slice note:** C2 prerequisite for BV-05.

- [x] **BK-05 — Feature metrics.** **Depends:** BK-01, BR-08. **Files:** feature metrics/specs. **Objective:** birth/age/activation/retirement, novelty, redundancy, consumers, marginal contributions, useful lineage, utility per byte/work, churn, dormancy, and obsolete protection. **Excludes:** human-semantic labeling as canonical utility. **Gate:** controlled lineage/activation fixtures yield exact metrics and correctly identify redundant/useful/dormant cases.

- [x] **BK-06 — Subproblem metrics.** **Depends:** BK-05. **Files:** subproblem metrics/specs. **Objective:** attained feature/intensity, original reward, stopping bonus/value, initiation/termination, learning progress, success, and cost. **Excludes:** assuming reward-oblivious tasks are reward-respecting. **Gate:** reward-respecting and reward-oblivious fixtures are distinguishable and fully traceable.

- [x] **BK-07 — Option metrics.** **Depends:** BK-06. **Files:** option metrics/specs. **Objective:** success, duration, return, stopping-state value/distribution, controllability, reliability, redundancy, planning participation, marginal gain, and acquisition/maintenance cost. **Excludes:** counting option creation as benefit. **Gate:** reliable/redundant/harmful/unused option fixtures receive expected classifications.

- [x] **BK-08 — Model and knowledge metrics.** **Depends:** BK-07. **Files:** model metrics/specs. **Objective:** one-step and option-horizon prediction, reward/stopping calibration, jump-length error, drift/recovery, uncertainty, harmful planning, and semantic stability. **Excludes:** comparing errors across changed targets without lineage alignment. **Gate:** controlled stale, biased, calibrated, and representation-shift models yield expected results.

- [x] **BK-09 — Planning metrics and consequential-backup test.** **Depends:** BK-08, D-19. **Files:** planning metrics/counterfactual runner. **Objective:** measure updates, states/options, search control, value gain per operation, realized agreement, primitive-time depth, latency/backups saved, exploitation failure, unused plans, and exact/approximate consequentiality. **Excludes:** “value changed” as sufficient consequence. **Gate:** exact tabular paired counterfactuals identify backups that do and do not alter later policy/action; approximation error is reported.

- [x] **BK-10 — Utility estimator hierarchy.** **Depends:** BK-05–BK-09, D-20. **Files:** utility engine/spec. **Objective:** implement exact leave-one-out, matched ablation attribution, consumer credit, and influence proxy with tier labels and cost accounting. **Excludes:** proxy-only C3 verdicts. **Gate:** proxy estimators are calibrated against exact diagnostic cases; sign errors and confidence failures force indeterminate utility.

- [x] **BK-11 — Discovery-cycle health metrics.** **Depends:** BK-05–BK-10. **Files:** cycle metrics/specs. **Objective:** forward rates, backward-credit latency/magnitude, survival by utility, generations, bottlenecks, useful/total growth, maintenance/benefit, collapse, runaway growth, cycling, and ossification. **Excludes:** artifact count as open-endedness. **Gate:** synthetic healthy/collapse/runaway/cycling/ossified traces are correctly separated. **Slice note:** required by BE-04; supports honest C3/C4 readiness without claiming C4.

- [x] **BK-12 — Failure-criteria detectors.** **Depends:** BK-02–BK-11. **Files:** failure rules. **Objective:** operationalize every charter section 14 failure with tolerance, window, comparator, and evidence requirement. **Excludes:** converting failure into missing evidence. **Gate:** one positive and one negative fixture per failure criterion; unavailable input yields indeterminate, not pass.

- [x] **BK-13 — Statistical aggregation and multiplicity.** **Depends:** BK-01, D-15. **Files:** statistical engine/spec. **Objective:** paired seed schedules, effect sizes, bootstrap intervals, preregistered outcomes, comparison families, Holm adjustment, missing/failed runs, and sensitivity reporting. **Excludes:** post-hoc seed removal or metric selection. **Gate:** golden statistical corpus matches an independent reference implementation and detects undeclared exclusions. **Slice note:** required by BV-05 for C2/C3 candidacy.

### 4.2 M2b — BRDC-1 reference cycle

- [ ] **BE-04 — Specify BRDC-1 without OaK overclaim.** **Depends:** D-17, BK-05–BK-11. **Files:** `docs/reference/BRDC-1-SPEC.md`. **Objective:** define public-ingredient feature candidates, reward-respecting subproblems, options, option models, planning, backward utility credit, scheduler events, and curation. **Excludes:** unpublished Oak Lab algorithms, deep/open-ended success claims. **Gate:** traceability table distinguishes public basis, BONSAI design choice, and experimental hypothesis for every mechanism.

- [ ] **BE-05 — Implement BRDC-1 feature and subproblem stages.** **Depends:** BE-04, BR-07. **Files:** reference agent. **Objective:** produce/revise/retire features and pose reward-respecting feature-attainment subproblems with full lineage/work telemetry. **Excludes:** counting creation as utility. **Gate:** diagnostic feature/subproblem fixture reconstructs exact lifecycle and cost; no labels enter Track A.

- [ ] **BE-06 — Implement BRDC-1 option and model stages.** **Depends:** BE-05. **Files:** reference agent. **Objective:** solve selected subproblems into options and learn option consequences online, batch one, no replay. **Excludes:** offline convergence passes. **Gate:** known tabular fixture reaches declared behavior/model tolerance in one pass and reports failure honestly when it does not.

- [ ] **BE-07 — Implement BRDC-1 planning and backward utility credit.** **Depends:** BE-06, BK-09–BK-10. **Files:** reference agent. **Objective:** use primitive/option models in planning and return consumer evidence to upstream artifacts under an external budget. **Excludes:** allowing internal scheduler self-report to enforce compliance. **Gate:** exact diagnostic counterfactual traces show forward construction and slower backward credit; planning work and consequences reconcile.

- [ ] **BE-08 — Implement artifact curation and lifecycle closure.** **Depends:** BE-07, BQ-05. **Files:** reference agent curation. **Objective:** retain/deprioritize/replace/remove artifacts based on a declared estimator while external governor stays authoritative. **Excludes:** claiming the estimator is the OaK solution. **Gate:** useful/redundant/stale fixtures produce expected dispositions and preserve immutable lineage/history.

- [ ] **BE-09 — Reference control and comparator adapters.** **Depends:** BE-02, BE-08. **Files:** comparator adapters/manifests. **Objective:** implement fixed-feature, primitive-only, no-planning, local-only utility, random/age retention, frozen representation, reward-oblivious subtask, primitive-model-only, bounded replay, dense-update, oracle, and unconstrained-growth controls. **Excludes:** merging Track B/C/D results with A. **Gate:** each comparator differs in exactly the intended mechanism and receives correct track/claim eligibility. **Slice note:** supplies the controlled ablations C3 requires; full matched-budget orchestration (BE-15) stays parked. **OD-03 SETTLED 2026-08-23 PT:** keep full eleven comparators (accept default).

### 4.3 M2c — C3 adjudication

- [ ] **BV-04 — C0 and C1 adjudication.** **Depends:** BQ-04, BM-04, BV-01. **Files:** claim rules/fixtures. **Objective:** require valid provenance/event/resource evidence and enforceable budget compliance with availability qualifications. **Excludes:** C1 pass when a declared hard counter was unavailable. **Gate:** compliant, soft-degraded, hard-violating, unavailable, tampered, and ambiguous-track bundles get exact verdicts.

- [ ] **BV-05 — C2 and C3 adjudication.** **Depends:** BV-04, BK-04–BK-10, BE-09, BK-13. **Files:** claim rules/fixtures. **Objective:** require continual adaptation and positive marginal abstraction utility under controlled ablation and statistical rules. **Excludes:** proxy-only utility, final-score-only adaptation, or comparator-track leakage. **Gate:** synthetic and reference diagnostic corpora cover pass/fail/indeterminate for every prerequisite.

---

## 5. ID mapping (v0.1 ↔ v0.2)

| v0.2 roster ID | v0.1 ID | Mapping |
|---|---|---|
| BQ-06 | BQ-06 | 1:1 identical |
| BK-04–BK-13 | BK-04–BK-13 | 1:1 identical |
| BE-04–BE-09 | BE-04–BE-09 | 1:1 identical |
| BV-04–BV-05 | BV-04–BV-05 | 1:1 identical |

No `M2S-*` renames are introduced. Session aliases are optional and non-canonical.

---

## 6. Completion criteria for the science slice

The M2 useful-abstraction science slice is complete only when:

1. All roster prompts above are `[x]` with DEVLOG entries, verification records, and focused commit SHAs.
2. BRDC-1 produces reconstructible lineage: feature → subproblem → option → model → planning, plus backward utility credit and curation dispositions.
3. Exact diagnostic utility evidence exists (leave-one-out / omit-one-event and/or matched ablation attribution); proxy-only paths cannot force C3 pass.
4. BV-05 emits machine C2/C3 verdicts in `{pass, fail, indeterminate}` with reason graphs and rule versions stored.
5. At least three diagnostic worlds (per **OD-01 SETTLED 2026-08-23 PT:** three BONSAI diagnostic worlds under BE-01 exercising distinct charter-family properties; BE-10–14 parked) have been run under matched instrumentation and declared seeds appropriate to the claim tier attempted.
6. README / reports / claim matrix cells touched by this slice honestly state limitations: no OaK reproduction; no instrument completion; no C4/C5 claim from this slice alone.
7. Parked items in §8 remain parked unless revived by addendum + explicit authority.

**Non-goals of slice completion:** physical OS energy backends, dense/event-driven scheduler superiority, full ten-family scenario suite, BE-15 matched-budget planner for all eleven charter ablations at acceptance scale, C4/C5 adjudication, long-duration L runs, security hardening beyond existing runtime isolation, offline-restore release packaging, or public announcement.

---

## 7. Explicit non-authorization

| Action | Status under this approved plan |
|---|---|
| Writing / reviewing / approving this PSPR; recording OD settlements | Allowed (still not STS) |
| Editing v0.1 checkboxes to match future progress | Allowed only as honest status append after authorized execution |
| Starting BQ-06 or any later prompt | **Forbidden** until `run it STS` / `run M2-science STS` / named-prompt STS |
| Claiming C3 pass for BRDC-1 | Forbidden until BV-05 gate + evidence |
| Claiming OaK reproduction | Always forbidden |
| Pushing to `main` as “execution complete” without gates | Forbidden |
| Reviving parked M3/M4 work | Requires addendum + explicit authority |

---

## 8. Parked ledger for this slice

Items below are **out of scope** for PSPR v0.2 M2-science execution. They inherit revival rules from `docs/governance/PARKED-SCOPE-LEDGER.md` plus the slice-local rows.

### 8.1 Carry-forward global parks (unchanged)

P-01…P-09 from the live parked-scope ledger remain parked (E3 probes, cluster/cloud, robotics, AMD/Intel collectors, hostile sandbox, neural mandate, Oak Lab reproduction, SaaS/telemetry upload, Track A human labels).

### 8.2 Slice-local parks (deferred from remaining v0.1 roster)

| ID | Parked item | Rationale | Revival |
|---|---|---|---|
| PS-01 | BQ-07–BQ-12 (fail-closed platform enforcement, decision replay, dense/event schedulers, scheduler conformance) | Not required to deliver lineage utility, BRDC-1, or machine C3 on diagnostic worlds | M3 PSPR cut + STS |
| PS-02 | BM-05–BM-14 (live OS backends, energy collectors, calibration closure) | Science slice may use existing BM-01–BM-04 portable/simulated paths with honest availability | M3 physical-host STS |
| PS-03 | BK-14 (full three-OS metric reproducibility/performance gate) | M2a needs correctness fixtures; cross-OS performance envelope is M3 | M3 STS |
| PS-04 | BE-10–BE-14 (full scenario-family implementations) | C3 diagnostic coverage via three BONSAI diagnostic worlds under BE-01 (**OD-01 SETTLED 2026-08-23 PT**); family enlargement is M3 | M3 STS |
| PS-05 | BE-15–BE-16 (full matched-budget ablation orchestration + A/L preregistration freeze) | BE-09 supplies comparators; acceptance-scale orchestration is M3/M4 | M3/M4 STS |
| PS-06 | BV-06–BV-10 (C4/C5, claim matrix generator, viz suite, cross-platform equivalence, overhead acceptance) | Beyond useful-abstraction / C3 slice | M3/M4 STS |
| PS-07 | BV-11–BV-16 (threat model hardening, offline restore, long L runs, docs handoff, release candidate) | Instrument completion / release path | M4 STS + separate publication authority |
| PS-08 | Acceptance A and Long L resource profiles as claim-required runs | D-16 A/L parked for this slice; S/C diagnostic runs only | M3/M4 STS |
| PS-09 | Public marketing claims, package-registry upload, release tags | Separate publication authority even if code is public | Explicit publication phrase |

---

## 9. Risks specific to this slice

| ID | Risk | Mitigation in roster |
|---|---|---|
| R-S1 | BRDC-1 mistaken for Oak Lab reproduction | BE-04 traceability table; report/claim wording gates |
| R-S2 | Proxy utility drives spurious C3 pass | BK-10 + BV-05 exclude proxy-only C3 |
| R-S3 | Comparator leakage into Track A | BE-09 track eligibility + BV-05 excludes |
| R-S4 | “Three families” ambiguous vs parked BE-10+ | **OD-01 SETTLED 2026-08-23 PT** (three diagnostic worlds under BE-01; BE-10–14 remain parked) |
| R-S5 | Replay retained under BRDC-1 | BQ-06 before claim-eligible BRDC-1 runs |
| R-S6 | Prior Continue-to-STS confused with v0.2 authority | **OD-02 SETTLED 2026-08-23 PT:** prior Continue-to-STS does not cover v0.2; §0 and handoff require fresh `run M2-science STS` / `run it STS` after approval |

---

## 10. Approval record

| Field | Value |
|---|---|
| Plan status | **APPROVED v0.2** 2026-08-23 (PT) by Basho Parks |
| OD settlements | **OD-01–OD-03 SETTLED 2026-08-23 (PT)** by Basho Parks accepting defaults (see handoff §4) |
| Execution authorization | **None from this approval** — plan approval ≠ STS; prior Continue-to-STS does not cover v0.2 (**OD-02**) |
| Required execution phrases | `run M2-science STS` or `run it STS` (or named milestone/prompt STS) |
| Suggested first prompt after STS | `BQ-06` (or parallel set in §3 if explicitly authorized) |
| Docs commit policy | Charter docs may land on `main` as plan authority; never treat docs commit as STS execution authority |

---

*End of BONSAI PSPR v0.2 (approved plan; not authorized for execution until STS).*
