# BONSAI PSPR v0.4 - M4 Acceptance and Release Candidate

**Benchmark for Online, Nonstationary, Single-pass Agent Intelligence**

Version: 0.4 (**APPROVED**)  
Date: 2026-08-24  
Status: **APPROVED v0.4** 2026-08-24 (PT) by Basho Parks; **FULL M4 STS AUTHORIZED** the same turn (preemptive: draft + approve + run without stopping)  
Author: drafted by Eustace; approved and STS-authorized by Basho Parks 2026-08-24 (PT)  
Authoritative local root: `C:\Users\17076\Documents\Reinforcement Learning Project`  
Remote: `USS-Parks/BONSAI-Research-Labs` (`main` @ `699d3eb` or later, public)  
Parent PSPR: [BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md](./BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md) (v0.1)  
Prior slices: v0.2 M2 (complete) · v0.3 M3 (complete on `main`)

> Plan approval and STS were given together: `Draft PSPR M4 next to execute BV-11 through BV-16 STS without stopping. I approve and authorize preemptively.`  
> Do not stop until BV-11–BV-16 are implemented, gated, committed, and on `main` with verified greens.  
> Publication / tags / registry upload remain **separately** unauthorized (BV-16 exclude).

---

## 0. Purpose and boundary

### 0.1 Goal

Deliver canonical **M4 — Acceptance and release candidate**:

1. **BV-11** Threat model and security boundaries  
2. **BV-12** Security, integrity, and denial-of-service hardening  
3. **BV-13** Dependency, supply-chain, and offline-restore gate  
4. **BV-14** Long-duration physical-host acceptance (L profile / 72h)  
5. **BV-15** Documentation, examples, and operator/researcher handoff  
6. **BV-16** Final evidence audit and release-candidate artifacts (no public upload)

### 0.2 Already complete (do not re-implement)

M0–M3 on Labs `main` (98 roster prompts checked). Last M3 code via PR #2; tip at draft time `699d3eb`.

### 0.3 Non-claims (hard)

| Claim | Status under v0.4 |
|---|---|
| OaK / Oak Lab reproduction | Forbidden |
| C4/C5 **pass** as trophy | Only if machine rules + evidence actually pass; do not force |
| Public upload, remote package registry, release tag | Forbidden without a later publication phrase |
| Hostile native-code sandbox | Forbidden (D-21 / P-05) |
| Invented 72-hour physical-host passes | Forbidden (**OD-01**) |

### 0.4 Universal verification gate

Same as v0.1 §0.6: verification log + fixtures/records; repo gates green; claim language cannot overstate.

---

## 1. Carry-forward

D-01–D-21 remain in force. Material: D-21 no hostile sandbox; D-16 L profile; D-09 public repo already true.

---

## 2. Sequential roster (6)

IDs 1:1 with v0.1. Order is dependency order.

| ID | Title | Depends | Gate (summary) |
|---|---|---|---|
| BV-11 | Threat model and security boundaries | BR-10, BQ-07, D-21 | `docs/security/THREAT-MODEL.md`; every trust boundary has controls, tests, residual risk, prompt mapping; sandbox disclaimer prominent |
| BV-12 | Security, integrity, DoS hardening | BV-11, BC-12, BR-03, BQ-06 | Validators, limits, hashing/signing, redaction, recovery; fuzz/property/adversarial suite; tamper/flood → bounded failure evidence |
| BV-13 | Dependency, supply-chain, offline-restore | BG-05, BV-12 | Policy, SBOM, vendor/offline scripts; offline/clean rebuild for required targets; no silent critical waivers |
| BV-14 | Long-duration physical-host acceptance | BE-16, BV-06–BV-13 | L manifests, host attestations, failure/recovery; 72h / 10M-step + 3 paired seeds per OS **or honest not-run** (**OD-01**) |
| BV-15 | Docs, examples, operator/researcher handoff | BV-09–BV-14 | Install/run/analyze/adapter/metric/claim/platform/limitations; new user can reproduce M1 and avoid overclaim |
| BV-16 | Final evidence audit and release candidate | all prior | Completion audit, notes, source/native/Python packages, hashes/SBOMs; independent bundle verify; **no** public upload/tag |

**First after STS:** `BV-11`. **Terminal:** `BV-16`.

---

## 3. Open decisions — SETTLED 2026-08-24 (PT)

| ID | Settlement |
|---|---|
| **OD-01** | BV-14 72h physical hosts: implement L manifests, attestation schema, recovery/failure tests, and CI-scale duration probes. If required Windows/macOS/Linux **physical** 72h hosts are not available this cycle, record **honest not-run / indeterminate** host evidence. Do **not** invent 72h passes or substitute one short CI job as L acceptance. Instrument-completion language must stay honest if L evidence is missing. |
| **OD-02** | Prior M3 STS does **not** cover v0.4. This document + Basho’s preemptive phrase **is** the M4 STS authority. |
| **OD-03** | BV-16 builds release-candidate **artifacts** (packages, hashes, SBOM, notes). No git tag, no crates.io/pypi upload, no marketing claim, unless Basho later says a publication phrase. |
| **OD-04** | Public Actions greens on Labs `main` are the hosted gate. Do not reopen private billing. |

---

## 4. Parked (unchanged)

P-01–P-09 global parks. No A/L **claim** from missing 72h evidence. No publication.

---

## 5. Completion criteria

When BV-11–16 gates pass (with OD-01 honesty):

1. Threat model + hardening + offline/SBOM path exist and are tested.  
2. Operator docs let a new user reproduce M1 without overclaim.  
3. Release-candidate artifacts exist locally in-repo (hashes recorded).  
4. L-profile **harness** exists; 72h physical evidence is either real or explicitly not-run.  
5. README / charter language does not claim instrument completion if OD-01 left L incomplete.  
6. All six prompts have verification-log evidence; CI greens on `main`.

---

## 6. Approval record

| Field | Value |
|---|---|
| Plan status | **APPROVED v0.4** 2026-08-24 (PT) |
| Execution | **FULL M4 STS** authorized same turn — do not stop until BV-11–16 on `main` with greens |
| Phrase | `Draft PSPR M4 next to execute BV-11 through BV-16 STS without stopping. I approve and authorize preemptively.` |
| First prompt | BV-11 |
| Docs commit | Plan authority; implementation commits must still pass gates |

---

*End of BONSAI PSPR v0.4 (APPROVED + full M4 STS authorized 2026-08-24 PT).*
