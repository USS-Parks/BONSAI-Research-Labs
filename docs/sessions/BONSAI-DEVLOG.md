# BONSAI development log

This append-only log records executed PSPR prompts. Corrections are added as new dated notes; prior entries are not silently rewritten. A prompt's implementation commit is the focused commit named below. When a prompt creates or changes this log, its own immutable SHA is appended by the next closeout entry because a commit cannot contain its final self-hash.

## 2026-07-18 — BG-01 — Establish independent repository identity

- Status: passed
- Authorization scope: user-authorized M0 STS, BG-01 through BG-10
- Dependencies and source revision: PSPR v0.1 approval; baseline `7d0ab846e46a9f38c3bd017da4837bf254b76bdc`
- Objective and exclusions: establish independent BONSAI identity; no remote creation, publication, or parent history import
- Reuse classification: reuse approved charter package and pre-existing independent repository; no implementation invented
- Files changed: root `README.md`; PSPR status
- Decisions/addenda: pre-created repository treated as evidence, not automatic completion; isolated STS worktree used
- Verification summary: authoritative checkout and session worktree resolve to the BONSAI common Git directory; baseline equals then-current `origin/main`; six charter files indexed; zero parent/absolute indexed paths
- Evidence paths and SHA-256 hashes: pre-BG-06 bootstrap output retained in the STS session transcript; no output artifact file existed yet
- Commit SHA: `210024faf5a315a1381318a408b49b6ae48fd751`
- Risks/blockers/parked scope changed: R-02 repository-identity hazard closed for this checkout
- Next eligible prompt: BG-02 and BG-03

## 2026-07-18 — BG-02 — Freeze source-of-truth governance

- Status: passed
- Authorization scope: user-authorized M0 STS
- Dependencies and source revision: BG-01 at `210024faf5a315a1381318a408b49b6ae48fd751`
- Objective and exclusions: encode authority, STS boundary, addenda, statuses, and history; charter science unchanged
- Reuse classification: reuse PSPR section 0 and global PSPR governance pattern; implement a repository-local docs checker
- Files changed: `docs/governance/SOURCE-OF-TRUTH.md`, `scripts/check_docs.py`, root `README.md`, PSPR status
- Decisions/addenda: M0 ends at BG-10; later prompts and external actions remain separately authorized
- Verification summary: eight Markdown files checked; local links resolved; explicit STS warnings present in root README and PSPR
- Evidence paths and SHA-256 hashes: pre-BG-06 bootstrap output retained in the STS session transcript; no output artifact file existed yet
- Commit SHA: `7193f224aa00ab2cbafeec0809ac618ea93dce6f`
- Risks/blockers/parked scope changed: none
- Next eligible prompt: BG-03 and BG-07

## 2026-07-18 — BG-03 — Adjudicate D-01 through D-21

- Status: passed
- Authorization scope: user-authorized M0 STS
- Dependencies and source revision: BG-01 at `210024faf5a315a1381318a408b49b6ae48fd751`
- Objective and exclusions: record accepted defaults, rejected alternatives, consequences, ownership, and supersession; no architecture implementation
- Reuse classification: reuse approved D-01 through D-21 exactly; group related decisions into seven ADRs
- Files changed: ADR index, ADR 0001 through ADR 0007, `scripts/check_adrs.py`, PSPR status
- Decisions/addenda: all 21 decisions accepted; zero unresolved material decisions
- Verification summary: every D-ID mapped exactly once in the index and exactly once in ADR bodies; docs check passed
- Evidence paths and SHA-256 hashes: pre-BG-06 bootstrap output retained in the STS session transcript; no output artifact file existed yet
- Commit SHA: `51b95630399816e5428e8effa6ef7fc6870f7a6c`
- Risks/blockers/parked scope changed: R-03 resolved by approved decision set
- Next eligible prompt: BG-04 and BG-08

## 2026-07-18 — BG-04 — License, visibility, and publication policy

- Status: passed
- Authorization scope: user-authorized M0 STS; no publication was performed by this prompt
- Dependencies and source revision: BG-03 at `51b95630399816e5428e8effa6ef7fc6870f7a6c`
- Objective and exclusions: enact D-09 dual license and publication boundary; no remote creation or push
- Reuse classification: reuse canonical Apache-2.0 and MIT license texts; implement BONSAI-specific policy
- Files changed: `LICENSE-APACHE`, `LICENSE-MIT`, `CONTRIBUTING.md`, `SECURITY.md`, publication policy, license checker, root README, PSPR status
- Decisions/addenda: SPDX expression `MIT OR Apache-2.0`
- Verification summary: license checker found only approved license files; publication policy required explicit authorization plus secret/privacy and redaction review; docs check passed
- Evidence paths and SHA-256 hashes: pre-BG-06 bootstrap output retained in the STS session transcript; no output artifact file existed yet
- Commit SHA: `482f4d7d8023a5f509a82ddc21a11cd2e7c5e525`
- Risks/blockers/parked scope changed: R-16 controlled by publication policy
- Next eligible prompt: BG-05

## 2026-07-18 — BG-05 — Locked Rust and Python workspace scaffold

- Status: passed
- Authorization scope: user-authorized M0 STS
- Dependencies and source revision: BG-03 and BG-04
- Objective and exclusions: create locked compile/lint/type/test surfaces; no BONSAI domain behavior
- Reuse classification: reuse Rust stable, Cargo, Python, uv, Ruff, Pyright, and Pytest at standard seams
- Files changed: Rust/Python workspace manifests and locks, two minimal package roots, tests, ignore policy, root README, existing checker lint corrections, PSPR status
- Decisions/addenda: Rust 1.96.0 pinned; Python 3.12–3.14 supported; uv 0.11.29 used; exact dev dependencies locked
- Verification summary: clean Windows checkout passed Cargo format, strict Clippy, Rust tests, Ruff, strict Pyright, and Pytest; no ignored/expected-failure tests
- Evidence paths and SHA-256 hashes: `Cargo.lock` SHA-256 `84C410D11522EEC3BCBC822EC9C6B15B987F35B91402597886050B01FAA2F17B`; `uv.lock` SHA-256 `3C745B23FDB0DF09F26CD6652FF1207BA9C5FC7577955B67C594BEA7468E37AC`
- Commit SHA: `444eb6b446d2adf0d7ff34104ca6fb373cbbea2e`
- Risks/blockers/parked scope changed: dependency/offline acceptance remains scheduled for BV-13
- Next eligible prompt: BG-06 and BG-09

## 2026-07-18 — Governance addendum — Public repository target

- Status: passed
- Authorization scope: direct user authorization to commit and push all existing charter and M0 work to public `USS-Parks/BONSAI-Research-Labs` `main`, with no secrets
- Dependencies and source revision: BG-05 at `444eb6b446d2adf0d7ff34104ca6fb373cbbea2e`
- Objective and exclusions: supersede the repository URL and private-visibility part of D-09 for charter/M0 source only; no future releases, evidence bundles, credentials, datasets, or capability claims
- Reuse classification: approved PSPR addendum seam
- Files changed: repository-target addendum, current repository metadata, publication/contribution policy, ADR amendment notice, historical handoff supersession notice, docs/license checker corrections
- Decisions/addenda: `docs/governance/addenda/2026-07-18-public-repository-target.md`
- Verification summary: GitHub metadata reported public visibility, default `main`, and empty repository; docs, ADR, and license checks passed; origin changed without force or publication yet
- Evidence paths and SHA-256 hashes: live GitHub API and Git remote output retained in the STS session transcript
- Commit SHA: `840bd13c5b3a477eb3c93ef719cc9f649bb18884`
- Risks/blockers/parked scope changed: R-16 publication exposure active; mandatory no-secret scan added to M0 closeout
- Next eligible prompt: resume BG-06

## 2026-07-18 — BG-06 — DEVLOG and verification-log machinery

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user-authorized M0 STS
- Dependencies and source revision: BG-02 and BG-05; public-repository addendum at `840bd13c5b3a477eb3c93ef719cc9f649bb18884`
- Objective and exclusions: append-only records with command/platform/revision/dirty state/times/exit status/artifact hashes; no reconstruction of nonexistent historical output files
- Reuse classification: reuse Cargo xtask convention plus `serde`, `serde_json`, and RustCrypto `sha2`; implement at the repository-task seam
- Files changed: `bonsai-xtask`, Cargo alias and lock, verification fixture, DEVLOG, verification log, record specification
- Decisions/addenda: records use UTC Unix nanoseconds; working directory is sanitized; environment is not enumerated; literal redaction is supported
- Verification summary: universal Rust/Python gates, docs/ADR/license checks, and deliberate pass/fail fixtures passed; records distinguish exit 0 from exit 1 and configured secret text is absent
- Evidence paths and SHA-256 hashes: `docs/verification/BONSAI-VERIFICATION-LOG.md`; manual pass stdout SHA-256 `b6ef6807dd96d18b833474ad68e7a23a29e562a29c67afa71a59fb9a73df0068`; fail stderr SHA-256 `02d3485f24dab97508da47c674806f5ac2d27a6174433a22545a169a20e80d73`
- Commit SHA: pending; append the focused implementation SHA in the next prompt's closeout entry
- Risks/blockers/parked scope changed: output bounding and hostile-command containment remain later security work; xtask is governance tooling, not a sandbox
- Next eligible prompt: BG-07 after gate and commit

### BG-06 closeout note

- Focused implementation commit SHA: `369bad35ee1c7599569c3e6fb12fceab5332e7ab`
- Ledger rule: appended by BG-07 because the BG-06 commit could not contain its own immutable hash

## 2026-07-18 — BG-07 — Risk, blocker, and parked-scope ledgers

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user-authorized M0 STS
- Dependencies and source revision: BG-02 and BG-06 at `369bad35ee1c7599569c3e6fb12fceab5332e7ab`
- Objective and exclusions: seed governed risks, blockers, parked scope, and claim matrix; do not revive parked work
- Reuse classification: reuse PSPR sections 9 and 10 exactly, updating statuses only where completed M0 evidence resolves a blocker
- Files changed: risk/blocker register, parked-scope ledger, claim-to-evidence matrix seed, validator, DEVLOG, verification log, PSPR status
- Decisions/addenda: R-02 and R-03 resolved; R-13 remains a future blocker; R-16 is active for the authorized public push; P-01 through P-09 remain parked
- Verification summary: exact R-01–R-16 and P-01–P-09 coverage; every required owner/status/revival/authorization field present; negative missing-owner and missing-revival fixtures rejected; universal gates passed
- Evidence paths and SHA-256 hashes: `docs/verification/BONSAI-VERIFICATION-LOG.md`; ledger files under `docs/governance/`
- Commit SHA: pending; append the focused implementation SHA in the next prompt's closeout entry
- Risks/blockers/parked scope changed: statuses reconciled as stated; no parked item revived
- Next eligible prompt: BG-08

### BG-07 closeout note

- Focused implementation commit SHA: `98ed62cd393f9c4cf6927ec8ce0efaa85a732c3a`
- Ledger rule: appended by BG-08 because the BG-07 commit could not contain its own immutable hash

## 2026-07-18 — BG-08 — Canonical terminology, identifiers, and units

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user-authorized M0 STS
- Dependencies and source revision: BG-03 and BG-07 at `98ed62cd393f9c4cf6927ec8ce0efaa85a732c3a`
- Objective and exclusions: freeze epoch-1 names, IDs, tracks, budgets, claim/availability states, units, missingness, precision, and clock semantics; no metric formulas
- Reuse classification: reuse SI/IEC units, UUID/SHA-256 representations, charter/PSPR terms, and explicit availability rules
- Files changed: terminology/units document, JSON registry, registry validator, DEVLOG, verification log, PSPR status
- Decisions/addenda: numeric canonical storage uses integer ns/B/uJ/uW/count where applicable; ratios/rewards use finite binary64; missing is never zero; identifiers are opaque
- Verification summary: 17 terms, eight identifiers, and 14 numeric fields passed; duplicate-name and unitless-numeric negative fixtures failed as required; universal and governance gates passed
- Evidence paths and SHA-256 hashes: `schemas/registry/terminology-v1.json`; `docs/verification/BONSAI-VERIFICATION-LOG.md`
- Commit SHA: pending; append the focused implementation SHA in the next prompt's closeout entry
- Risks/blockers/parked scope changed: none
- Next eligible prompt: BG-09

### BG-08 closeout note

- Focused implementation commit SHA: `85e408def2e4e74ef472aa46d29ce4d44f8b677d`
- Ledger rule: appended by BG-09 because the BG-08 commit could not contain its own immutable hash

## 2026-07-18 — BG-09 — Three-OS hosted CI topology

- Status: passed
- Authorization scope: user-authorized M0 STS plus approved push to public `USS-Parks/BONSAI-Research-Labs` `main`
- Dependencies and source revision: BG-05, BG-06, and BG-08 at `85e408def2e4e74ef472aa46d29ce4d44f8b677d`
- Objective and exclusions: exercise baseline gates on Windows, macOS, and Linux and label hosted evidence; no physical, energy, or long-duration acceptance claim
- Reuse classification: reuse GitHub-hosted standard runners and pinned official checkout/upload actions plus pinned Astral uv setup action
- Files changed: baseline workflow, test matrix, CI topology validator, sanitized evidence writer, DEVLOG, verification log, PSPR executing status
- Decisions/addenda: Windows x86_64, macOS arm64 plus Intel, and Linux x86_64 hosted jobs; Python 3.12, Rust 1.96.0, uv 0.11.29; checkout credentials are not persisted
- Verification summary: local universal/governance gate passed; live push run 29669146969 passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel; all classification artifacts were inspected and deny physical/energy/long-duration claims
- Evidence paths and SHA-256 hashes: `docs/verification/TEST-MATRIX.md`; workflow `https://github.com/USS-Parks/BONSAI-Research-Labs/actions/runs/29669146969`; Linux `7139746343B454756FD8F293ACD77BAC453F9DC6FA13DBA71BAD8BB2E9BA1F88`; macOS arm64 `EFC562205BE6FFB843B38EECD909E6F673AFD595F1DEEDEC33EB813064601FFF`; macOS x86_64 `A65D4768622F798061BB0DEB61BC4201A45C14A03907FCAE133BCE72BD4AB97B`; Windows `5D31F7F1C40216C26F0FC07B3561FDA302AA6D3549F875630D94F872C4E8E909`
- Commit SHA: `59e474a8a3eeddbc071b02c0152d8d7925b9af27` (workflow checkpoint); this closeout note follows after live evidence
- Risks/blockers/parked scope changed: hosted evidence explicitly cannot resolve R-04, R-05, R-13, or R-14
- Next eligible prompt: BG-10
