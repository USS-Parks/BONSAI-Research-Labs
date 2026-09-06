# BONSAI PSPR v0.3 - M3 Cross-Platform Governed Science

**Benchmark for Online, Nonstationary, Single-pass Agent Intelligence**

Version: 0.3 (**APPROVED**)    
Date: 2026-08-24  
Status: **APPROVED v0.3** 2026-08-24 (PT) by Basho Parks; OD-01–OD-05 settled (accept defaults); **STS AUTHORIZED** 2026-08-24 (PT) via `run M3 STS` / full STS phrase  
Author: drafted by Eustace (OaK Labs support); approved by Basho Parks 2026-08-24 (PT)  
Authoritative local root: `C:\Users\17076\Documents\Reinforcement Learning Project`  
Remote: `USS-Parks/BONSAI-Research-Labs` (`main` @ `da7dfc9`, public as of 2026-08-24)  
Parent PSPR: [BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md](./BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md) (v0.1, approved 2026-07-18)  
Prior slice: [BONSAI-PSPR-v0.2-M2-USEFUL-ABSTRACTION.md](./BONSAI-PSPR-v0.2-M2-USEFUL-ABSTRACTION.md) (v0.2, APPROVED + STS complete on `main`)

> **Plan approval is not authorization to execute.**  
> Approving this draft does **not** start implementation.  
> No prompt in this roster may begin until Basho Parks says `run M3 STS` / `run it STS`, or explicitly authorizes a named milestone cut (`run M3a STS`, …) or named prompt IDs from this document.  
> Prior `run M2-science STS` / `Continue to STS` authority does **not** cover v0.3 (**OD-02 default**).

---

## 0. Purpose and boundary

### 0.1 User-chosen goal

Deliver the canonical **M3 — Cross-platform governed science** milestone:

1. **Live OS resource backends** — Windows / macOS / Linux measurement and enforcement with honest capability detection (BM-05–BM-10).
2. **Energy tiers and resource semantics** — E0–E3 adjudication, optional NVIDIA collector, physical qualification where hardware exists, cross-platform comparability matrix (BM-11–BM-14).
3. **Fail-closed platform governance + schedulers** — enforcement bridge, decision replay, dense vs event-driven schedulers under matched budgets (BQ-07–BQ-12).
4. **Full scenario families + orchestration** — BE-10–BE-16 families, matched-budget ablation orchestration, S/C pilots and preregistration freeze; BK-14 metric reproducibility/performance gate.
5. **Claim / equivalence path through BV-10** — C4/C5 adjudication rules, claim-to-evidence matrix, visualization/reporting, cross-platform equivalence suite, instrumentation-overhead acceptance (BV-06–BV-10).

This PSPR does **not** claim OaK reproduction, instrument completion, M4 release-candidate closure, hostile-sandbox certification, long-duration L acceptance as release-required evidence, package-registry upload, or marketing publication.

### 0.2 Relation to prior plans

| Item | Disposition |
|---|---|
| M0 / M1 / M2 (including v0.2 science STS) | Complete on Labs `main` @ `da7dfc9`. Do **not** re-implement. |
| v0.1 full roster (BG→BV-16) | Remains historical approved charter-to-prompt map. |
| Remaining M3 work (this document) | **Scoped and ordered here** as the proposed execution roster. |
| M4 (BV-11–BV-16) | **Parked** for a later PSPR (see §8). |

**Already complete (do not re-implement):**  
M0 BG-01–10; BC-01–12; M1 BR-01–06, BM-01–04, BQ-01–04, BK-01–03, BE-01–03, BV-01–03; M2 runtime BR-07–10, BQ-05–06; M2 science BK-04–13, BE-04–09, BV-04–05 (C3 machine adjudication).

### 0.3 Non-claims (hard)

| Claim | Status under v0.3 |
|---|---|
| OaK / Oak Lab algorithm reproduction | Forbidden |
| Instrument completion / release candidate | Forbidden (M4) |
| C4/C5 **pass** as project completion | Not required for M3 gate (canonical: C4 experiments can run but need not pass) |
| Uniform energy availability across OS | Forbidden — honest E0/E1 when unqualified |
| Hostile native-code sandbox | Forbidden (P-05 / D-21) |
| Publication / marketing / registry upload | Separate authority |

### 0.4 Universal verification gate

Identical to v0.1 §0.6 / v0.2: every prompt ends with evidence in the verification log, fixtures or physical-host records as required, repository gates green where applicable, and claim language that cannot overstate availability or enforcement.

### 0.5 Repo / infra context (2026-08-24)

| Fact | Note |
|---|---|
| Labs remote | `USS-Parks/BONSAI-Research-Labs` is **public** (flipped 2026-08-24) — standard hosted Actions minutes for public repos apply; private-minute burn no longer applies to this repo. |
| Charter remote | `USS-Parks/BONSAI` remains charter-only / not the build remote. |
| Local root | `C:\Users\17076\Documents\Reinforcement Learning Project` fast-forwarded to `da7dfc9`; working tree was clean at draft time. |
| Hosted M1 aggregate | Previously blocked on private-plan spending limits; re-evaluate after public flip (**OD-04**). |

---

## 1. Settled carry-forward decisions

D-01–D-21 from v0.1 remain in force unless a dated addendum overrides them. Especially material for M3:

| ID | Relevance to M3 |
|---|---|
| D-09 | Public repository target already in force (addendum 2026-07-18); Labs public as of 2026-08-24. |
| D-13 | First-release accelerator: NVIDIA via NVML where present; Apple integrated GPU only via documented path. |
| D-14 | Energy E0–E3 honesty; no invented zeros. |
| D-15 / D-16 | Resource profiles S/C vs A/L; A/L claim runs stay gated (see OD + parks). |
| D-21 | Hostile native-code sandbox excluded from v1. |

---

## 2. Milestone cuts for this slice only

Canonical M3 gate (v0.1): *Required physical-host conformance gates pass; platform availability differs honestly; C4 experiments can run but need not pass.*

| Cut | Focus | Prompt IDs | Cut gate (usable artifact) |
|---|---|---|---|
| **M3a** | Live OS measure + enforce | BM-05–BM-10 | Each OS has a backend with capability matrix; intentional violation fixtures show hard limit or documented monitor/terminate fail-closed behavior |
| **M3b** | Energy + semantics | BM-11–BM-14 | E0–E3 adjudicator + comparability matrix; unqualified backends stay honest |
| **M3c** | Platform governor + schedulers | BQ-07–BQ-12 | Fail-closed enforcement bridge; dense vs event schedulers compared under matched streams |
| **M3d** | Scenario families + freeze | BE-10–BE-16, BK-14 | Ten family implementations + matched-budget orchestration + S/C pilot freeze artifact; metric perf/repro gate |
| **M3e** | Claims through overhead | BV-06–BV-10 | C4/C5 rules executable; claim↔evidence matrix; cross-platform equivalence + overhead acceptance recorded |

Default STS after approval authorizes **M3a→M3e in dependency order** unless Basho narrows the phrase (**OD-03**).

---

## 3. Dependency order (minimal)

```text
BM-05 ─┬─ BM-06 ─┐
BM-07 ─┼─ BM-08 ─┼─ BQ-07 ─ BQ-08
BM-09 ─┴─ BM-10 ─┘     │
                       ├─ BQ-09 ─ BQ-10 ─ BQ-11 ─ BQ-12
BM-05/07/09 ─ BM-11 ─ BM-12 ─ BM-13 ─ BM-14
BK-02…13 (done) ─ BK-14
BE-09 (done) ─ BE-10…BE-13
BE-09 + BQ-07 + BK-08 ─ BE-14
BE-10…14 + BQ-12 + BK-13 ─ BE-15 ─ BE-16
BV-05 (done) + BK-11…13 (done) + BE-15/16 + BM-12 ─ BV-06 ─ BV-07
BK-14 + BV-02 + BE-15 ─ BV-08
BM-14 + BK-14 + BE-03 + BV-07 ─ BV-09
BM-04 + BK-03 + BE-16 ─ BV-10
```

Parallelism allowed only where depends are already green and Basho/STS explicitly authorizes a parallel set.

---

## 4. Sequential prompt roster (29)

IDs and objectives are **1:1 with v0.1** remaining M3 prompts. Do not invent new ticket prefixes.

### 4.1 M3a — Live OS backends (BM-05–BM-10)

| ID | Title | Depends | Gate (summary) |
|---|---|---|---|
| BM-05 | Windows measurement backend | BM-04 | Live Windows physical-host calibration vs controlled workloads / Job Object accounting |
| BM-06 | Windows enforcement primitive spike | BM-05, D-21 | Intentional hard-limit crossings; unsupported limits declared |
| BM-07 | macOS measurement backend | BM-04 | Live Apple-silicon calibration; privilege/unsupported fields explicit |
| BM-08 | macOS enforcement primitive spike | BM-07 | Enforcement or documented fail-closed monitor/terminate with measured overshoot |
| BM-09 | Linux measurement backend | BM-04 | Live cgroup v2 reconciliation; nested trees |
| BM-10 | Linux enforcement backend | BM-09 | Controller behavior, cleanup, no descendant escape |

### 4.2 M3b — Energy and resource semantics (BM-11–BM-14)

| ID | Title | Depends | Gate (summary) |
|---|---|---|---|
| BM-11 | NVIDIA accelerator collector | BM-01, BM-04 | Feature-detect supported/not-supported/no-permission; absence never breaks CPU-only |
| BM-12 | Energy evidence tiers + calibration metadata | BM-04, BM-05, BM-07, BM-09, BM-11 | E0–E3 fixtures cannot overclaim |
| BM-13 | Physical energy backend qualification | BM-12 | Qualification records or honest unqualified E0/E1 |
| BM-14 | Cross-platform resource semantics audit | BM-05–BM-13 | Three-platform availability/comparability matrix |

### 4.3 M3c — Governor bridge + schedulers (BQ-07–BQ-12)

| ID | Title | Depends | Gate (summary) |
|---|---|---|---|
| BQ-07 | Fail-closed platform enforcement integration | BM-06, BM-08, BM-10, BQ-03 | Unsupported hard controls reject before Track A |
| BQ-08 | Governance-decision replay | BQ-02–BQ-07, BR-09 | Reconstruct admissions/violations from immutable policy + records |
| BQ-09 | Scheduler event and eligibility contract | BR-04, BQ-05 | Schema + instrumentation for eligibility/queues |
| BQ-10 | Dense scheduler comparator | BQ-09 | Wake every eligible component; charge all work |
| BQ-11 | Event-driven scheduler candidate | BQ-09, BQ-10 | Declared eligibility, deferral/drop, work charging |
| BQ-12 | Scheduler matched-budget conformance | BQ-10–BQ-11, BK-03 | Dense vs event under identical streams |

### 4.4 M3d — Families, orchestration, metric gate (BE-10–BE-16, BK-14)

| ID | Title | Depends | Gate (summary) |
|---|---|---|---|
| BE-10 | Stable dynamics / changing values family | BE-09 | Diagnostic + enlarged scenarios for revaluation |
| BE-11 | Observation aliasing and late-factor families | BE-09 | Aliasing vs representation insufficiency separated |
| BE-12 | Long-life plasticity and noisy single-pass families | BE-09, BK-04 | Plasticity vs forgetting distinguished |
| BE-13 | Temporal-joints and recursive-reuse families | BE-09, BK-07–BK-09 | Recursive reuse without ambiguity |
| BE-14 | Distractor, resource-shock, model-mismatch families | BE-09, BQ-07, BK-08 | Manifest-visible shocks; detectors localize |
| BE-15 | Matched-budget ablation orchestration | BE-10–BE-14, BQ-12, BK-13 | All eleven charter ablations have runnable pairs |
| BE-16 | Resource-profile pilots and preregistration freeze | BE-15, D-15–D-16 | Freeze artifact separates exploratory vs confirmatory |
| BK-14 | Metric reproducibility and performance gate | BK-02–BK-13 | Deterministic derivation + bounded memory/streaming envelope |

### 4.5 M3e — Claims through overhead (BV-06–BV-10)

| ID | Title | Depends | Gate (summary) |
|---|---|---|---|
| BV-06 | C4 and C5 adjudication | BV-05, BK-11–BK-13, BE-15–BE-16, BM-12 | Rules/fixtures; pass not required for M3 close |
| BV-07 | Claim-to-evidence matrix generator | BV-04–BV-06 | Every claim maps to rules/metrics/fixtures |
| BV-08 | Visualization and comparison reporting | BK-14, BV-02, BE-15 | Lifelong curves, uncertainty, resource fronts |
| BV-09 | Cross-platform equivalence suite | BM-14, BK-14, BE-03, BV-07 | Schema/manifest/track/ordering equivalence matrix |
| BV-10 | Instrumentation-overhead acceptance | BM-04, BK-03, BE-16 | Paired no/minimal/full instrumentation deltas |

**First eligible prompt after full M3 STS:** `BM-05` (or authorized parallel BM-05/07/09).  
**Terminal prompt for this PSPR:** `BV-10`.

---

## 5. ID mapping (v0.1 → v0.3)

All IDs are unchanged from the approved v0.1 roster. v0.3 does not renumber. Checkbox state in the canonical roster should be updated only as prompts gate on `main`.

---

## 6. Completion criteria for M3

When M3a–M3e gates pass:

1. Live backends exist for Windows, macOS, and Linux with capability matrices and violation evidence.
2. Energy/accelerator paths adjudicate E0–E3 honestly; missing hardware does not invent evidence.
3. Dense and event-driven schedulers are comparable under matched budgets; platform enforcement fails closed.
4. Full scenario-family set + matched-budget orchestration + preregistration freeze artifact exist.
5. C4/C5 rules are executable; cross-platform equivalence and instrumentation overhead are measured and reported.
6. README / claim language remains non-overclaiming (no OaK reproduction; no instrument completion; no implied uniform energy).
7. M4 work (BV-11–BV-16) remains parked.

---

## 7. Explicit non-authorization

| Action | Forbidden until |
|---|---|
| Implementing any v0.3 prompt | Fresh STS phrase after plan approval |
| Claiming C4/C5 pass as M3 success | Explicit claim authority + evidence |
| Treating docs-only commit as STS | Never |
| Reviving M4 / P-01–P-09 parks | Addendum + named authority |
| Publication / marketing / registry upload | Separate publication phrase |
| Privileged host reconfiguration | Explicit equipment/collector authority |

---

## 8. Parked ledger for this slice

### 8.1 Carry-forward global parks (unchanged)

P-01–P-09 from `docs/governance/PARKED-SCOPE-LEDGER.md` remain parked.

### 8.2 Slice-local parks (deferred from remaining v0.1 roster)

| ID | Parked item | Rationale | Revival |
|---|---|---|---|
| PS-10 | BV-11–BV-16 (threat model, hardening, SBOM/offline restore, long L acceptance, docs handoff, release candidate) | Instrument completion / release path | **M4 PSPR** + STS (+ publication authority for BV-16 packaging) |
| PS-11 | Acceptance A and Long L as **claim-required** runs | D-16; M3 may pilot S/C and prepare freeze; A/L claim runs stay gated | M4 STS or explicit A/L claim authority |
| PS-12 | Public marketing claims, package-registry upload, release tags | Separate publication authority even though Labs is public | Explicit publication phrase |
| PS-13 | E3 laboratory-grade energy probes beyond adapter seam | P-01 | PSPR addendum + equipment authority |
| PS-14 | AMD/Intel discrete accelerator collectors | P-04 | PSPR addendum + collector authority |
| PS-15 | Hostile sandbox / certification | P-05 / D-21 | Dedicated security addendum |

---

## 9. Risks specific to this slice

| ID | Risk | Mitigation |
|---|---|---|
| R-M3-1 | Physical hosts unavailable → M3 blocked | OD-01; allow honest capability gaps; do not fake live gates |
| R-M3-2 | Energy/NVIDIA overclaim | BM-12/13 + D-14; absence stays E0/E1 |
| R-M3-3 | False cross-OS numeric equivalence | BM-14 + BV-09 distinguish semantic vs performance equivalence |
| R-M3-4 | Scheduler work charged incorrectly | BQ-12 matched-stream conformance |
| R-M3-5 | BE family expansion mistaken for OaK reproduction | Carry BE-04 traceability discipline; claim wording gates |
| R-M3-6 | Prior M2 STS confused with M3 authority | OD-02: fresh `run M3 STS` / `run it STS` required |
| R-M3-7 | Full 29-prompt STS too large / thrash | OD-03: allow `run M3a STS` first cut |
| R-M3-8 | CI noise on public matrix | OD-04: re-check hosted aggregate after public flip |

---

## 10. Open decisions (for Basho)

| ID | Decision | Draft default (accept unless overridden) |
|---|---|---|
| **OD-01** | Which physical hosts are in-scope for M3 live gates this cycle? | **Default:** Windows 11 x86_64 (Basho workstation) + Linux x86_64 (CI and/or local) + macOS arm64 (CI and/or available Apple host). Missing host → capability `unsupported` with documented gap, not invented pass. |
| **OD-02** | Does prior M2 STS cover any v0.3 work? | **Default:** **No.** Fresh STS required after approving v0.3. |
| **OD-03** | Authorize full M3 (29) or phased first STS? | **Default:** Plan approval covers full M3 roster; **first STS phrase may be** `run M3a STS` (BM-05–10) unless Basho says `run M3 STS` for the whole cut. |
| **OD-04** | Hosted M1 semantic-equivalence / aggregate jobs after public flip | **Default:** Re-run matrix on `main` after approval commit; treat green public Actions as sufficient; do not re-open private spending-limit work unless Basho asks. |
| **OD-05** | NVIDIA / energy qualification this cycle | **Default:** Implement BM-11–13 interfaces + fixtures; qualify only on hardware actually present; otherwise remain E0/E1 without blocking CPU-only M3 close. |

---

## 11. Approval record

| Field | Value |
|---|---|
| Plan status | **APPROVED v0.3** 2026-08-24 (PT) by Basho Parks |
| OD settlements | **OD-01–OD-05 SETTLED** 2026-08-24 (PT) by Basho Parks accepting defaults (full M3 STS) |
| Execution authorization | **FULL M3 STS** authorized 2026-08-24 (PT) — do not stop until all 29 prompts gated; commit/push to main with verified greens |
| Required plan phrase | `approve PSPR v0.3` (or equivalent clear approval) |
| Required execution phrases after approval | `run M3 STS` / `run it STS` / `run M3a STS` / named prompt STS |
| Suggested first prompt after full M3 STS | `BM-05` (or parallel BM-05/07/09 if authorized) |
| Docs commit policy | After approval, charter docs may land on `main` as plan authority; docs commit ≠ STS |

---

## 12. Project next-steps map (beyond this STS)

| Horizon | What | Trigger |
|---|---|---|
| Immediate | Approve/amend v0.3; settle OD-01–OD-05; optional docs commit to `main` | Basho plan phrase |
| Near | M3a backends STS → physical-host evidence | `run M3a STS` or `run M3 STS` |
| Mid | M3b–M3e through BV-10 | Continues under same approved roster + STS scope |
| Later | **PSPR v0.4 M4** acceptance/release (BV-11–BV-16) | New draft after M3 close |
| Parallel (non-roster) | Keep Labs public; charter repo identity separate; no marketing claims without publication authority | Standing |

---

*End of BONSAI PSPR v0.3 (APPROVED + full M3 STS authorized 2026-08-24 (PT)).*
