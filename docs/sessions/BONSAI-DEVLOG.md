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

## 2026-07-18 — BG-10 — M0 baseline and governance checkpoint

- Status: passed
- Authorization scope: user-authorized M0 STS; BC-01 remains unauthorized
- Dependencies and source revision: BG-01 through BG-09; BG-09 closeout at `d2774154931f91a3205ac415aed7c791cddd5035`
- Objective and exclusions: reconcile files, decisions, risks, gates, logs, milestone cuts, publication, and claim boundaries before contract code; no BC implementation
- Reuse classification: reuse every M0 checker and ledger through one source-of-truth checkpoint
- Files changed: M0 checkpoint, M0 validator, DEVLOG, verification log, PSPR executing status
- Decisions/addenda: M0 ends with a governed foundation and explicit stop before BC-01; C0–C5 and instrument completion remain not-run
- Verification summary: pre-commit and clean-commit universal/governance/M0 gates passed; clean checkpoint scanned 13 commits with Gitleaks 8.30.1 and found zero leaks; final closeout remote equality and hosted CI are attested externally because a commit cannot contain its own remote SHA/run identity
- Evidence paths and SHA-256 hashes: `docs/verification/M0-CHECKPOINT.md`; `docs/verification/BONSAI-VERIFICATION-LOG.md`; command output retained in the STS session transcript
- Commit SHA: `1b68656057a6920f5a087e03d1ca181f914b2791` (implementation checkpoint); this closeout commit marks the prompt complete
- Risks/blockers/parked scope changed: no parked scope revived; R-16 controlled by the final no-secret scan and non-force push
- Next eligible prompt: none within current authorization after BG-10

## 2026-07-18 — BC-01 — Schema-version policy and compatibility harness

- Status: passed
- Authorization scope: user instruction `Continue to STS`, expanding execution from the published M0 checkpoint through the remaining approved roster in dependency order
- Dependencies and source revision: BG-08 and BG-10 complete; published M0 checkpoint `8873e13444512a5035f45527c6cacff5d14301e5`
- Objective and exclusions: freeze epoch/minor evolution, field reservation, canonical JSON, compatibility fixtures, and migration obligations; no BONSAI domain message
- Reuse classification: reuse official Protobuf evolution rules and the existing Cargo xtask/Serde/SHA-256 seam; reuse Lamprey Harness `scripts/verify-proof.cjs` at MIT revision `d9d53786ca71550861883a61bf8088b43e3275d8` as a fail-fast proof-command pattern only; no Lamprey runtime or source copied
- Files changed: Protobuf and JSON policy READMEs, Rust schema checker and canonicalizer, five candidate fixtures plus baseline, machine verification records and immutable-artifact Git attributes, PSPR execution history/reuse ledger/status, DEVLOG, verification log
- Decisions/addenda: epoch paths use `v<epoch>` and catalog versions use `epoch.minor`; minor changes are additive; deleted Protobuf fields reserve number and name; numeric semantics include units; breaking changes require new immutable migrated output while retaining original bytes
- Verification summary: the first recorded universal attempt retained a tool-path failure after Rust/schema success; the second used checksum-verified uv 0.11.29 and passed format, Clippy, Rust tests, Ruff, Pyright, Pytest, schema compatibility, docs, ADR, license, governance-ledger, terminology, and CI-topology checks; additive fixture passed and all four prohibited changes failed with their intended codes
- Evidence paths and SHA-256 hashes: `evidence/verification/records.jsonl`; pass stdout `4b08b7548416b12819e7796578ca19101d65ab4183f4947a51912373e85a4462`; pass stderr `d90189be1ef824c4bf3053e76efa053cf5fd78b771b0a73db70bacf28fae7020`; fixture digests are printed in the retained pass artifact
- Commit SHA: pending this prompt's immutable implementation commit; the next closeout entry will append it under the log's self-hash rule
- Risks/blockers/parked scope changed: no new blocker; compatibility catalog is test tooling, not a domain schema; physical-system claims remain out of scope
- Next eligible prompts: BC-02, BC-03, and BC-04; dependency order selects BC-02

## 2026-07-18 — BC-02 — Universal event envelope

- Status: passed
- Authorization scope: user instruction `Continue to STS`; approved roster in dependency order
- Dependencies and source revision: BC-01 implementation and closeout at `a0b4aba07191d8035330bec4f0eeb0bf64bb31e8`; hosted run 29670167856 passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel
- Objective and exclusions: define and validate the universal epoch-1 event envelope and cross-language relay; wall time is never treated as global order
- Reuse classification: implement at the existing `bonsai-contracts` seam using official Protobuf semantics, Prost 0.14.4, vendored protoc 3.2.0, SHA-256, and Python protobuf 7.35.1; one generated descriptor is shared across Rust and Python; no Lamprey runtime/source reused
- Files changed: epoch-1 envelope proto, contracts build/code/examples/tests, Python cross-language conformance test and dependency lock, Rust lock, Protobuf/root/source-of-truth docs, CI schema step, PSPR status, machine verification evidence, DEVLOG, verification log
- Decisions/addenda: IDs are nonzero 16-byte UUID representations; source sequence and monotonic time are source/clock-domain facts; causal parents form partial order; payload bytes are bound to SHA-256; the supported unknown-field relay validates known fields then returns original binary bytes unchanged
- Verification summary: focused Rust/Python gate passed; two recorded attempts retained Windows self-lock failures caused by running `bonsai-xtask.exe` while its integration test rebuilt that same executable; an identical temporary verifier copy then passed the full universal/governance gate; Python → Rust → Python preserved appended field 99 byte-for-byte; invalid ID, zero monotonic time, negative wall time, and hash mismatch fixtures failed closed
- Evidence paths and SHA-256 hashes: `evidence/verification/records.jsonl`; pass stdout `1a318078e6b2ff263443932379c4d30f16d4448d4101b816b38271503131a081`; pass stderr `764edc6682fe3a2cc67bdbedbc6c3353e3ec768ef2dfb7892df148c4551f3c9b`; envelope `A515C37F366EE16C58DC82608493F58FDFE6C66E251F384318EB40E610B8FAA1`
- Commit SHA: pending this prompt's immutable implementation commit; the next closeout entry will append it under the log's self-hash rule
- Risks/blockers/parked scope changed: R-15 remains controlled by generated bindings, pinned locks, and schema gate; Protobuf serialization is not claimed canonical; opaque relay is the only unknown-field-preserving Rust path
- Next eligible prompts: BC-03 and BC-04; dependency order selects BC-03

### BC-02 closeout note

- Focused implementation commit SHA: `127b20b68957fb1473fba670fe4cd411187c062e`
- Hosted verification: GitHub Actions run 29670584785, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BC-03 because the BC-02 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-18 — BC-03 — Experiment manifest schema

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user instruction `Continue to STS`; approved roster in dependency order
- Dependencies and source revision: BC-01 at `a0b4aba07191d8035330bec4f0eeb0bf64bb31e8`, BG-08 at `85e408def2e4e74ef472aa46d29ce4d44f8b677d`, and BC-02 closeout at `127b20b68957fb1473fba670fe4cd411187c062e`
- Objective and exclusions: define one immutable, fully resolved experiment-manifest contract for source identity/dirty state, adapter/environment configuration, explicit seeds, declared track/replay facts, resource profile, metrics, scenario, expected counters, and pre-run publication eligibility; no mutable runtime defaults and no actual track/resource-policy derivation
- Reuse classification: extend the existing canonical JSON/SHA-256 and Cargo xtask seam; reuse the standards-conformant `jsonschema` 0.48.1 Draft 2020-12 validator with network/file resolution disabled; implement the BONSAI manifest schema and fixtures as new domain contracts; no Lamprey runtime/source reused
- Files changed: experiment-manifest schema and four fixtures, schema checker/docs, pinned Rust dependency lock, root status, PSPR status, DEVLOG, verification log, and retained machine evidence
- Decisions/addenda: seed values are explicit canonical decimal strings; adapter/environment configuration objects are always present and fully resolved; replay is always declared; E0 requires a null energy budget while E1–E3 require a positive budget; dirty sources require a patch hash and are publication-ineligible; eligibility never authorizes publication; BC-05 still derives track and BC-06 still defines detailed policy/decisions
- Verification summary: Draft 2020-12 meta-schema validation, no-default audit, valid fixture, LF/CRLF canonical equivalence, and three required-declaration rejections passed; two recorded universal Cargo/uv/schema/governance invocations passed through a byte-identical external verifier copy to avoid the known Windows target self-lock, and the final ledger/status tree passed the full gate again without recording a third machine entry
- Evidence paths and SHA-256 hashes: machine records `BC-03-1784430332012813700` and `BC-03-1784430432949968400`; initial stdout/stderr `44068a1b16625c008f56bf0793a650c797fa30781bbcfef9cba31156a8004518` / `f3f69f47d929d3395e12716ade68822210a2e922c6cf58ee723ecd0664ccfc0a`; confirmation stdout/stderr `c217a429e089398bd0d7d4e02dd4993dbfa4e6fb283bb4b36b424620eabce721` / `d07e8f3303112a7a301dfe1e290bcac2c67666a972a65ee56479fb9df68d599a`; canonical manifest `dc596b67136ae83046831e381cf0a5deab0719d54e874c5c26facc95ce140f57`; canonical schema `e4942f9d6a254cb31c574c8899b4d0814b6e421c38a0c9f889b1c1f61dd4a523`; verifier `7D56967E130ED5EFF5372F4B7AE908A126429FDEE170B4E74DA4C80DCCAEB735`; `Cargo.lock` `F2565497C1C59EBB1C22F88FCA096A0D05E1EFD9435F99D46C71E4DCFDF17D22`
- Commit SHA: pending; append the focused implementation SHA in the next prompt's closeout entry
- Risks/blockers/parked scope changed: R-15 remains controlled by the versioned schema, pinned validator, canonical hashes, and schema gate; no physical behavior, enforcement, energy fidelity, instrument completion, or C0–C5 result is claimed
- Next eligible prompt: BC-04

### BC-03 closeout note

- Focused implementation commit SHA: `d31e4a6e8126697357e7f0870f434ee24881e664`
- Hosted verification: GitHub Actions run 29671499350, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BC-04 because the BC-03 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-18 — BC-04 — Platform and dependency inventory

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user instruction `Continue to STS`; approved roster in dependency order
- Dependencies and source revision: BC-01 complete at `a0b4aba07191d8035330bec4f0eeb0bf64bb31e8`; BC-03 published and hosted-green at `d31e4a6e8126697357e7f0870f434ee24881e664`
- Objective and exclusions: define a strict sanitized inventory for OS/build/architecture, CPU/GPU class, memory, clocks, drivers, runtimes/compilers, dependency locks, privilege, collectors, and thermal/power state plus collector-boundary Rust types; no live host probe, collector installation, secret retention, serial-number retention, physical evidence, or energy-fidelity claim
- Reuse classification: extend the existing Draft 2020-12 validator/canonical hash seam and `bonsai-contracts`; reuse Serde already locked by BC-03; implement new inventory types and boundary sanitizer at the existing contracts seam; no new external dependency and no Lamprey runtime/source reused
- Files changed: platform-inventory schema, raw/sanitized fixtures, Rust inventory types/sanitizer/tests, schema gate/docs, Cargo manifests/lock, root/PSPR status, BC-03 hosted closeout, DEVLOG, verification log, and machine evidence
- Decisions/addenda: sanitized machine identity is an independently assigned opaque UUID, never a hostname/serial hash; forbidden identity/path/credential fields are removed recursively before strict decoding; public structs deny unknown fields; collector status and privilege requirements remain explicit; unavailable thermal state is recorded, never converted to nominal or zero
- Verification summary: focused Rust tests and strict Clippy passed; raw sensitive input was invalid for public use; exact sanitized output validated through the Rust contract and Draft 2020-12 schema; hostname, Windows/Linux user paths, CPU/GPU serials, registry/collector tokens, and API key were absent while required reproducibility fields remained; full universal/schema/governance gate passed through the byte-identical external verifier copy
- Evidence paths and SHA-256 hashes: machine record `BC-04-1784431710849423900`; stdout `6236464b9c190f686ab4bda19163caa493fc2f222dd5b69fe34dc94c4aac09f1`; stderr `2d341ac1bc78080ced2b174604a62a4fe54707bf6599f1c81a856d6f9d9763b7`; sanitized inventory canonical SHA-256 `0bb0b95eaa8d0440c417a316b09a3694658036cd48592a8fc21ef7b8ac975514`; schema canonical SHA-256 `f1fd3f59feab9ebbf6c06581d0011371ba8e4a7d68eb10e202bdbe4ad55830b5`; verifier `F81144D316A5EB77F682E13535F20C0DCF53AE723171C82C7FD53FFCC6FB7AEF`; `Cargo.lock` `BA7C806E67A42D80EC8D0D0D9781F937BEDA17B2B2B291C333BFDE5FFE9ABA04`
- Commit SHA: pending; append the focused implementation SHA in the next prompt's closeout entry
- Risks/blockers/parked scope changed: R-04/R-05/R-14 remain active for later live backends and privileged collectors; BC-04 records capability/availability contracts only and does not weaken their physical gates
- Next eligible prompt: BC-05

### BC-04 closeout note

- Focused implementation commit SHA: `a694e2380b907d04aea41bca321bb091f6c2ba28`
- Hosted verification: GitHub Actions run 29671931286, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BC-05 because the BC-04 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-18 — BC-05 — Track and replay declarations

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user instruction `Continue to STS`; approved roster in dependency order
- Dependencies and source revision: BC-03 complete; BC-04 published and hosted-green at `a694e2380b907d04aea41bca321bb091f6c2ba28`
- Objective and exclusions: derive mutually exclusive A/B/C/D or `INDETERMINATE_TRACK` from complete runtime capability/data-flow facts; never accept self-attested Track A
- Reuse classification: extend `bonsai-contracts` and the existing Draft 2020-12 schema gate; no new dependency or Lamprey runtime/source
- Files changed: track schema, seven-case corpus, Rust declaration/verdict types and classifier/tests, schema docs/gate, BC-04 closeout, PSPR/root status, logs, and machine evidence
- Decisions/addenda: observer-data access is an indeterminate boundary violation; privileged inputs derive D; replay/offline updates derive B; dense scheduling derives C; only complete batch-one/single-pass/fixed-budget facts derive A; declaration mismatch is retained
- Verification summary: all seven classification fixtures passed; strict Clippy initially rejected the independent boolean fact surface, then passed after one documented contract-local exception preserving contradictory-fact visibility; full universal/schema/governance gate passed
- Evidence paths and SHA-256 hashes: machine record `BC-05-1784432466868264500`; stdout `f64546cc170ae1c2139942911b5e5b061f94d209e201d41179dd64e3e7b892b0`; stderr `9f28fbdc232e471eed34de59792aff947d843343f19bc7bd794a02b145c39f49`; schema `eefeeba41b7a875c02bb6f5104ad6f02f2d3c16582594c57c4e669c798e6f2fa`; verifier `27F9E6B316D53BA2C379C7CC96176CE670B99440D173D0D7B6BF28FF034C8FC3`
- Commit SHA: pending; append in the next prompt under the self-hash convention
- Risks/blockers/parked scope changed: R-07 remains controlled by later runtime isolation; this prompt defines classification only
- Next eligible prompt: BC-06

### BC-05 closeout note

- Focused implementation commit SHA: `eaa0e52ec5a6dc78ab1a360f2a11c2201c7a5e9d`
- Hosted verification: GitHub Actions run 29672261289, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BC-06 because the BC-05 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-18 — BC-06 — Resource policy and governor decisions

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user instruction `Continue BONSAI PSPR execution BC-06`; approved roster prompt BC-06
- Dependencies and source revision: BC-03 and BG-08 complete; BC-05 published and hosted-green at `eaa0e52ec5a6dc78ab1a360f2a11c2201c7a5e9d`
- Objective and exclusions: define immutable resource-policy JSON and governor-decision Protobuf contracts for all four budget scopes, nine work classes, distinct soft/hard limits, measured/estimated basis, and admit/defer/throttle/reject/terminate outcomes; no counter collection, scheduler, budget arithmetic, or backend enforcement
- Reuse classification: extend the existing Draft 2020-12, canonical JSON, generated Protobuf, and `bonsai-contracts` validation seams; no new dependency and no Lamprey runtime/source reused
- Files changed: resource-policy schema/fixture and semantic validator, governor decision Protobuf and reconstruction validator/tests, generated-contract build input, schema gate/docs, BC-05 hosted closeout, PSPR/root status, DEVLOG, verification log, and retained machine evidence
- Decisions/addenda: policy identity/version plus canonical SHA-256 binds exact policy bytes; every canonical work class has an explicit allocation and every limit is allocated exactly once; rolling-window evidence carries start/duration; measured/estimated observations require a present numeric value, estimated observations require estimator identity/version, and unavailable observations require a reason while prohibiting a numeric value; outcome-specific action fields fail closed; reason-code semantics live in the versioned policy; no enforcement is claimed
- Verification summary: focused tests and schema gate passed; the first recorded universal attempt retained a sandbox denial for uv's user-profile cache after Rust/contract success; redirecting the unchanged uv 0.11.29 cache to ignored repository `target` storage produced full passes across format, Clippy, 12 Rust contract tests, Python lint/types/tests, schema compatibility, docs, ADR, license, governance-ledger, terminology, and CI-topology checks; the final recorded pass includes the unavailable-without-zero hardening; all five decision outcomes preserved exact policy/observed/request/reason reconstruction inputs
- Evidence paths and SHA-256 hashes: final machine pass `BC-06-1784434736275348400`; pass stdout `fd7fb2cbddb47a2b64f862baf0665b1b9b62e63182e80a6ccf638b0cb9fd9a45`; pass stderr `7c992754329533ffa7fd6c392ea75036966df312bbcf39be27e6a8158698d8b2`; policy fixture canonical `5053b8c5b78e46d1bf45b542815598f5fd127981ed61f1311938879badc77b49`; policy schema canonical `d2bc586d01c69ee7f1202ef3d8f324692661b6ecc8266c514ba0f25b2f32e877`; decision proto raw `9DF0DC65708FEABB81818F7237CEA86DD97C7E9422370A9EBEAB58062B96AB12`; verifier `87A82D2A9C0C663BF63FF634C118AAAB2DA7AA17D322A66B496707A1CC4CF733`
- Commit SHA: pending; append in the next prompt under the self-hash convention
- Risks/blockers/parked scope changed: no parked scope revived; R-04/R-05/R-14 remain active for later live measurement/enforcement; the retained uv cache denial is an environment record and was resolved without weakening a gate
- Next eligible prompt: BC-07
