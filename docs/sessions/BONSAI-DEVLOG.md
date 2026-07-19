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

### BC-06 closeout note

- Focused implementation commit SHA: `5542580c2f9870fa5f6d539a402b6577f898ca0e`
- Hosted verification: GitHub Actions run 29673221983, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BC-07 because the BC-06 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-19 — BC-07 — Cognitive-artifact and lineage schemas

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user instruction `Continue to STS PSPR BC-07`; approved roster prompt BC-07
- Dependencies and source revision: BC-02 published and hosted-green at `127b20b68957fb1473fba670fe4cd411187c062e`; BG-08 complete at `85e408def2e4e74ef472aa46d29ce4d44f8b677d`; BC-06 published and hosted-green at `5542580c2f9870fa5f6d539a402b6577f898ca0e`
- Objective and exclusions: define epoch-1 Protobuf and registry contracts for all seven cognitive-artifact types, stable identity, immutable revisions, provenance parents, consumer links, cost/utility history, and nonterminal/terminal dispositions; do not prescribe artifact representations, learning algorithms, metric formulas, claim verdicts, or implement the BR-07 runtime registry
- Reuse classification: extend the existing generated-Protobuf/event-availability and `bonsai-contracts` validation seams; reuse the BG-08 artifact types and identifiers unchanged; extend the frozen terminology registry with BC-07 lifecycle vocabulary and units; implement a pure conformance state model with no new dependency
- Files changed: artifact/lineage Protobuf, generated-contract build input, Rust lineage conformance validator/property tests, terminology registry/checker and governance docs, Protobuf/schema docs, BC-06 hosted closeout, PSPR/root status, DEVLOG, verification log, and retained machine evidence
- Decisions/addenda: parent provenance is an acyclic DAG distinct from the potentially cyclic consumer graph; revision ancestry uses an exact preceding immutable revision ID and does not create a self-parent edge; retained/deprioritized are nonterminal while replaced/retired/removed are terminal; replacement names an already born new identity; delayed cost/utility evidence may be appended after terminal disposition without permitting revision or relinking; unavailable values never become numeric zero; BC-08 remains authoritative for formal metric-result semantics
- Verification summary: all seven artifact types passed a root lifecycle; a complete trace covered consumer, measured cost, estimated utility, retained disposition, and revision; generated property cases rejected every tested wrong predecessor, birth/revision provenance omission, forbidden parent cycles of lengths two through six, and same-identity revision after replaced/retired/removed while accepting a new identity with terminal-parent provenance; strict Clippy and 18 Rust contract tests passed; the full universal/schema/governance gate passed; the initial retained invocation failed before checks because local PowerShell script execution was disabled, and the unchanged script passed under process-local `-ExecutionPolicy Bypass`
- Evidence paths and SHA-256 hashes: final redacted machine pass `BC-07-1784448872886496000`; pass stdout `ea1a9a532166f8250d24638690a3661be9b26cc7545cfd67386de2db07c9117b`; pass stderr `7449e8d77f278cdd99ea805f2a921dccb5ab89a7feeaffdacb1f060f366c0cb8`; retained redacted environment failure `BC-07-1784448859471416000` with stderr `9d2e0504f9d61e37fdda21eeb6ecb435a7feb8655f30ffb5ed7e82d7a7fced4c`; lineage Protobuf `CC52D7E4FFDC9888224990C6952BF7729A2072739E4C1AC67C6338E61BC2B553`; terminology registry `C1F9D6BB0DFCD283D8B7116FA04502352FEEFEA907718C89CF30213131485A39`; terminology checker `9E6180C1C5FA53D063EA31FAD3DB97F3E509F5D5AD2CFAE948161B454FD268D0`; unchanged `Cargo.lock` `BA7C806E67A42D80EC8D0D0D9781F937BEDA17B2B2B291C333BFDE5FFE9ABA04`
- Commit SHA: pending; append in the next prompt under the self-hash convention
- Risks/blockers/parked scope changed: no parked scope revived; R-08 and R-12 remain active for later runtime ingestion/lineage hardening, and BC-07 makes no agent-utility, physical-host, instrument-completion, or C0–C5 claim
- Next eligible prompts: BC-08, BC-09, and BR-01; dependency order selects BC-08

### BC-07 closeout note

- Focused implementation commit SHA: `9d0bd38b9a4b1aa1bce1823fd0a2f42a0dd755c4`
- Publication authorization: later gated PSPR source publication authorized by direct user instruction during BC-08; recorded in `docs/governance/addenda/2026-07-19-later-pspr-source-publication.md`
- Hosted verification: GitHub Actions run 29679768852, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BC-08 because the BC-07 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-19 — BC-08 — Metric, uncertainty, and claim-result schemas

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user instruction `STS BC-08`; later gated PSPR source publication to public `USS-Parks/BONSAI-Research-Labs` `main` authorized during execution
- Dependencies and source revision: BC-01 complete at `a0b4aba07191d8035330bec4f0eeb0bf64bb31e8`; BG-08 complete at `85e408def2e4e74ef472aa46d29ce4d44f8b677d`; BC-07 published and hosted-green at `9d0bd38b9a4b1aa1bce1823fd0a2f42a0dd755c4`
- Objective and exclusions: define immutable Draft 2020-12 metric specification, metric estimate, uncertainty, and claim-result contracts requiring formula/version/unit/direction, population, window, estimator, missingness, precision, uncertainty, hashed inputs, criteria, evidence, and reasoned verdicts; do not compute a metric, adjudicate a claim, or hard-code any C0–C5 pass
- Reuse classification: extend the existing Draft 2020-12, canonical JSON/SHA-256, and `cargo xtask schema-check` seams; reuse BG-08 units/availability/claim states and the pinned `jsonschema` dependency; implement four new contracts and a coherent fixture matrix with no new dependency
- Files changed: four JSON schemas, nine metric/claim fixtures, schema checker/tests, schema docs, continuing-source publication addendum/policy, BC-07 commit/hosted closeout, PSPR/root status, DEVLOG, verification log, and retained redacted machine evidence
- Decisions/addenda: specifications bind formula and estimator parameter hashes; scalar estimates always carry unit plus nonempty hashed provenance while unavailable/excluded results carry a reason and no invented scalar; uncertainty is a separate immutable record bound to estimate identity/unit; claim results require a versioned/hashed criterion, nonempty evidence and reason codes, explicit prerequisites, and pass/fail/indeterminate/not-run; the fixture suite binds estimate to exact spec hash and uncertainty to exact estimate, validates all four claim states, and requires the interval to contain the fixture estimate; BK-01 remains metric-computation authority and BV-01 remains claim-adjudication authority; continuing source publication is recorded by dated addendum
- Verification summary: all four schemas passed Draft 2020-12 meta-validation, prohibited-default audit, valid fixture validation, and LF/CRLF canonical equivalence; cross-record identity, version, hash, unit, interval, estimate, uncertainty, and claim-evidence links passed; all four claim states validated with nonempty reasons/evidence; scalar-without-unit, scalar-without-provenance, verdict-without-criterion, verdict-without-evidence, and verdict-without-reason fixtures failed with exact stable codes; strict Clippy, workspace tests, Python gates, schema compatibility, and all governance checks passed in the retained full gate
- Evidence paths and SHA-256 hashes: machine pass `BC-08-1784449953957691900`; redacted stdout `c232171ce1fd1e7401f81b31bd88a61e1d1dd12131920bb1d83a7c058dd7e971`; redacted stderr `32ec1870131da130bcce39794f00a6795438fc632b95f672e93507c91ed9d29e`; metric-spec fixture/schema canonical `c61b496da6e23b4722f1b2cb6097faa30857ec95815bac6120a2564f54351b09` / `bc242866c9afe1bc5e5733a016a8d7b83c2ba440baecd2842a0339cf2cce55f8`; estimate fixture/schema canonical `e184a2f810b46537bb1fe4e313ab2cbba7b145ddeafd1db62d0bc6100277379b` / `696303bf3474ea0460f6a09729b685f5e7d23ab37e2d448a315cbd854ebeb87d`; uncertainty fixture/schema canonical `73ade7bc3f440ca906a2de691bab968d88d1dd2e7f624cac1ec497a0eeb89d6f` / `86bf3d1138f51e739d7fa11eb59da1d0be96f2d5762de0f54230b5367313554b`; claim fixture/schema canonical `15f0cb08e252710f0e762475abbd84dda004a38e71b93d483a430bc9919ef5eb` / `cbd4a294664277e4443b733753de455358881fdc533e8a6c03b93c85e26fbf15`; unchanged `Cargo.lock` `BA7C806E67A42D80EC8D0D0D9781F937BEDA17B2B2B291C333BFDE5FFE9ABA04`
- Commit SHA: pending; append in the next prompt under the self-hash convention
- Risks/blockers/parked scope changed: no parked scope revived; R-10 and R-12 remain active until numeric cross-platform and utility-estimator calibration gates, R-15 remains controlled by schema/version fixtures, and R-16 remains controlled by the new target-specific source-publication addendum plus redaction/no-slop gates
- Next eligible prompts: BC-09 and BR-01; dependency order selects BC-09

### BC-08 closeout note

- Focused implementation commit SHA: `1fd36aec8c4379ef594cb8cf18ea9be035af7870`
- Hosted verification: GitHub Actions run 29680114105, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BC-09 because the BC-08 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-19 — BC-09 — Append-only event segment format

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user instruction `Commence STS BC-09. Authorized all for entire session.` plus the existing later-gated public source publication addendum
- Dependencies and source revision: BC-02 published and hosted-green at `127b20b68957fb1473fba670fe4cd411187c062e`; BC-08 published and hosted-green at `1fd36aec8c4379ef594cb8cf18ea9be035af7870`
- Objective and exclusions: implement bounded length-delimited event segment frames, checksummed immutable headers/frames/footers, contiguous segment sequence, crash-safe no-clobber finalization, validation, and deterministic recovery; no event-semantic ingestion validation, arbitrary mutation, in-place truncation/compaction, index, blob store, or SQLite authority
- Reuse classification: implement the approved ADR 0003 append-only Protobuf-segment seam as new `bonsai-bundle` code; reuse the already locked SHA-256, Serde/JSON, and tempfile dependencies at their standard seams; no Lamprey runtime/source reused and no new production dependency introduced
- Files changed: `bonsai-bundle` crate and integration tests, committed event-segment outcome matrix, event-segment format specification, workspace manifest/lock, BC-08 hosted closeout, PSPR/root status, DEVLOG, verification log, and retained machine evidence
- Decisions/addenda: format epoch 1 uses checksummed fixed headers, opaque bounded frames, and terminal footers; bundle sequence starts at zero and is contiguous; the hard frame ceiling is 16 MiB while every segment declares a lower or equal immutable bound; finalization synchronizes staging bytes before atomic no-clobber publication and removes staging only after the final file is synchronized; recovery never truncates or rewrites the staged source, copies complete unfinalized frames to a separate recovery file, leaves partial/corrupt evidence untouched, and fails closed on final-path conflict; BR-03 retains event-semantic validation authority and BC-10 retains index/blob authority
- Verification summary: the focused tests produced exact stable outcomes for valid segments, truncated headers/frames, frame and segment checksum corruption, oversized frames, duplicate/non-monotonic sequences, complete-open recovery, already-finalized recovery, and partial-frame refusal; strict focused Clippy and docs governance passed; the first recorded full gate retained a Windows self-lock failure when the running in-tree `bonsai-xtask.exe` was rebuilt by workspace tests; a SHA-256-identical external verifier copy ran the unchanged full gate successfully across format, strict workspace Clippy, 25 Rust tests plus the verification fixture, Python lint/types/tests, schema compatibility, docs, ADR, license, governance-ledger, terminology, and CI-topology checks
- Evidence paths and SHA-256 hashes: environment failure `BC-09-1784451381921850200` with stderr `3d41c471b558a533d892d35b698f389a026bded8199be1255933b82a461efb74`; passing machine record `BC-09-1784451458102382400` with stdout `ce492dcd4830a385675cc9a9f83211a03a3c2e6bdfb2ffecce8b4be4158f1f5f` and stderr `248e453649141d134e331cf7315a372ee49cf95cfa6b7723f00c3cbf8e3d0413`; fixture matrix `6D4E776A41E0F13817F6EB91E5D7AAA218CD672BDF88FF41CFAC285331520AEB`; implementation `CE1201DC152AAA06C44EE99246FC746827B978A2578B342A8393971884471293`; verifier `BD1174BB50222583463195CEA005EA5B9A411AF56C5DE05CDF84DF2676CDB5A6`; `Cargo.lock` `2B8C7C3C5687B4717246AB688EC6700C22FE42CA190053A4248ACFBCC7B302A9`
- Commit SHA: pending; append in the next prompt under the self-hash convention
- Risks/blockers/parked scope changed: R-08 and R-09 remain active for later transport/ingest/backpressure and observer-reserve work; R-15 is controlled here by the explicit epoch, checksums, immutable format specification, and fixture corpus; no parked scope revived and no physical-host, instrument-completion, or C0–C5 claim is made
- Next eligible prompts: BC-10 and BR-01; dependency order selects BC-10

### BC-09 closeout note

- Focused implementation commit SHA: `26c093df265a3ae96089201140c74149ffd93caf`
- Hosted verification: GitHub Actions run 29680948955, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BC-10 because the BC-09 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-19 — BC-10 — Portable bundle index and content-addressed blobs

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user instruction `Commence STS BC-10. Authorized all for entire session.` plus the existing later-gated public source publication addendum
- Dependencies and source revision: BC-03 through BC-09 published; immediate predecessor BC-09 hosted-green at `26c093df265a3ae96089201140c74149ffd93caf`
- Objective and exclusions: implement a portable SQLite metadata index over immutable event segments and content-addressed derived-artifact blobs while keeping files authoritative; no network database, agent access, mutable event history, arbitrary filesystem path, or analytical Arrow/Parquet contract
- Reuse classification: extend the existing `bonsai-bundle` validation, SHA-256, crash-safe publication, tempfile fixture, and ADR 0003 seams; add the current `rusqlite` 0.40.1 release with bundled SQLite as the only new production dependency; implement the version-1 migration, typed blob identity/store, rebuild, and read-only query layer without a parallel storage authority
- Files changed: embedded SQLite migration, bundle-index/blob implementation and integration tests, committed exact-outcome matrix, storage-format documentation, workspace lock, BC-09 commit/hosted closeout, PSPR/root status, DEVLOG, verification log, and retained machine evidence
- Decisions/addenda: exact bytes define a typed SHA-256 blob identity and the only accepted path is `blobs/sha256/{2 hex}/{62 hex}.blob`; writes synchronize staging bytes and publish without clobber; identical existing content is idempotent while wrong expected hashes and corrupt/colliding targets fail explicitly; symlinks, noncanonical encodings, path escape, and path-tampered index rows fail closed; the disposable `STRICT` SQLite schema uses application ID 1112429385, user version 1, canonical decimal text for unsigned 64-bit fields, full-file hashes, and no authoritative payload; rebuild validates/hashes every segment and blob before publishing a fresh index; the public handle uses `SQLITE_OPEN_READ_ONLY` plus `query_only`; BC-11 retains analytical derivation authority
- Verification summary: the committed matrix produced exact outcomes for file-only rebuild, supplied hash mismatch, corrupt/colliding target, traversal-bearing blob identity, traversal-bearing index row, and read-only open; a second rebuild after deleting the index reproduced every segment/artifact row; a separate SQLite read-only handle rejected mutation; focused strict Clippy and Rust tests passed; the retained full gate passed formatting, strict workspace Clippy, 29 Rust tests, 2 Python tests, schema and verification fixtures, schema compatibility, docs, ADR, license, governance-ledger, terminology, and CI-topology checks on Windows x86_64
- Evidence paths and SHA-256 hashes: passing machine record `BC-10-1784470096942208100`; stdout `df787b8c90b703ae89a11c763340abe47227b501eef6c7a5fac9d9230703997a`; stderr `b9760d1c9f45a39f3acb9deba9891470198394138a23e009222d51e0e30387e5`; fixture matrix `ED9EE03878CE64037321F24523465F121CC5AA37DD7AB37AE7250697CF2000BE`; implementation `95A8D70DB484E7323C8D3F84B9BBAF15FF492E2F64FC9A17D15D71F5B20AE471`; migration `BED4C1BB45E54115070D1174FD3D2871FA01F3DDC751705E279013EF68B9F40D`; verifier `E8BDEA495B4FDADCB63E16637023CFAC698BFB9C55043755FEBD0ADD65839E8F`; `Cargo.lock` `79715B2EC269F5C5CC3E4B0755D2BB7870DDC6C6A8EB15197BF5252EAC15CAE8`
- Commit SHA: pending; append in the next prompt under the self-hash convention
- Risks/blockers/parked scope changed: R-15 remains active for later format migration/offline restore and is controlled here by embedded migration identity, locked bundled SQLite, exact fixtures, and rebuildability; R-16 remains controlled by the continuing source-publication addendum plus redaction/no-slop gates; no parked scope revived and no agent, physical-host, instrument-completion, or C0–C5 claim is made
- Next eligible prompts: BC-11 and BR-01; dependency order selects BC-11

### BC-10 closeout note

- Focused implementation commit SHA: `5b64c01413abc4f7c6ae189e14e3e94d88380bb7`
- Hosted verification: GitHub Actions run 29690432057, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BC-11 because the BC-10 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-19 — BC-11 — Arrow/Parquet analytical derivation contract

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user instruction `STS BC-11 and BC-12 in this session, fully authorized.` plus the existing later-gated public source publication addendum
- Dependencies and source revision: BC-08 through BC-10 published; immediate predecessor BC-10 hosted-green at `5b64c01413abc4f7c6ae189e14e3e94d88380bb7`
- Objective and exclusions: implement deterministic Arrow schemas and provenance-bound Parquet materialization for event, metric, lineage, and resource-governor decision tables; no raw-evidence authority, event repair, metric computation, claim adjudication, or silent stale-table acceptance
- Reuse classification: extend the existing `bonsai-bundle`, typed SHA-256 identity, BC-08 metric/claim, BC-09 event, BC-10 content/index, and ADR 0003 seams; add current Apache Arrow/Parquet Rust 59.1.0 crates with default Parquet codecs disabled; implement typed rows, frozen schemas, semantic hashing, provenance metadata, create-new materialization, and validation without a second analytical authority
- Files changed: Arrow/Parquet dependencies and lock, analytical derivation implementation and integration tests, committed exact-outcome matrix, derivation-contract documentation, BC-10 commit/hosted closeout, PSPR/root status, DEVLOG, verification log, and retained machine evidence
- Decisions/addenda: epoch-1 tables use only Arrow `Utf8`, `UInt64`, and `Float64` with order/nullability bound into explicit schema hashes; unavailable metric values remain null and non-finite values fail; every Parquet file carries format, kind, schema hash, semantic hash, sorted unique authoritative-input hashes, producer ID/version, and row count; semantic identity hashes logical rows independently of Parquet page boundaries; validation reads all batches and recomputes schema/row/semantic identity; current source-set mismatch returns `DERIVATION_INPUT_MISMATCH`, producer or content staleness returns `DERIVATION_STALE`, and kind/schema failures remain distinct; materialization never replaces an existing derivative; BC-12 retains whole-bundle validity authority
- Verification summary: all four typed tables materialized and validated with exact schemas/provenance; independent regeneration from identical rows/inputs produced identical semantic summaries; wrong authoritative input and stale producer fixtures returned exact distinct outcomes; an existing derivative remained byte-unchanged after replacement refusal; focused strict Clippy/tests passed; the retained full gate passed formatting, strict workspace Clippy, 32 Rust tests, 2 Python tests, schema and verification fixtures, schema compatibility, docs, ADR, license, governance-ledger, terminology, and CI-topology checks on Windows x86_64
- Evidence paths and SHA-256 hashes: passing machine record `BC-11-1784471649324168400`; stdout `2612baef9494fab59c60ec5d92cc88d7514546bbdc963d2c6143c911afe73b09`; stderr `2ab885413b1a6ca9d0d624a9af41d2b9a25fab6fc2e7367985dea0fe034672c0`; fixture matrix `E0E03D4B39C7CA746C275F83A3B6919A174F5900910861CD5C2A2805FE371BE7`; implementation `12CC2CA0791C955517E23C66DB003FA77D7BB85F72D22576B208F211D2DF8349`; verifier `C021A501CFB1B84689457F8820866666A75B0C0726F5E0BBFA5FF5E8D942027E`; `Cargo.lock` `282632B786D3D1972A6B92379A47A658EBFBC97AFBB992C1B40C86B9F4C4FB0D`
- Commit SHA: pending; append in the next prompt under the self-hash convention
- Risks/blockers/parked scope changed: R-10 remains active for later numeric equivalence and is controlled here by exact logical semantic hashes plus hosted matrix validation; R-15 remains active for migration/offline restore and is controlled here by frozen schema/provenance identities and locked Arrow/Parquet versions; R-16 remains controlled by the continuing publication addendum and no-slop gate; no parked scope revived and no raw-authority, physical-host, instrument-completion, or C0–C5 claim is made
- Next eligible prompts: BC-12 and BR-01; dependency order and current authorization select BC-12

### BC-11 closeout note

- Focused implementation commit SHA: `7c483f3e0024da32163cc461c33d77162fc87156`
- Hosted verification: GitHub Actions run 29691229681, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BC-12 because the BC-11 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-19 — BC-12 — Bundle validator and migration conformance

- Status: local gate passed; initial hosted attempt exposed a Windows checkout-byte portability defect; correction pending focused commit and hosted rerun
- Authorization scope: user instruction `STS BC-11 and BC-12 in this session, fully authorized.` plus the existing later-gated public source publication addendum
- Dependencies and source revision: BC-01 through BC-11 published; immediate predecessor BC-11 hosted-green at `7c483f3e0024da32163cc461c33d77162fc87156`
- Objective and exclusions: produce one machine-readable whole-bundle report covering schemas, content hashes, derived track, inventory availability, resource policy, failures, metric provenance, and version migration; no scientific claim adjudication, evidence repair, unsupported-epoch interpretation, or mutation of source bundles
- Reuse classification: extend the existing `bonsai-bundle`, `bonsai-contracts`, SHA-256 identity, schema checker, track derivation, inventory/resource/metric contracts, and `bonsai-xtask` seams; reuse the already locked `jsonschema` crate; implement the typed manifest/report, deterministic v0 migration, forward read-only posture, CLI, and exact corpus without creating another evidence or claim authority
- Files changed: bundle-manifest and validation-report schemas, current/v0/forward and four negative/limited fixture families, typed validator and migration implementation, CLI and integration tests, schema gate, validation/migration documentation, BC-11 hosted closeout, PSPR/root status, DEVLOG, verification log, and retained machine evidence
- Decisions/addenda: current epoch 1 is fully interpreted; the only supported old epoch migrates deterministically in memory and reports the migrated-byte hash without rewriting source; future epochs expose only a stable header plus required-file hash verification and are `read_only`, with all current semantic checks explicitly `not_run`; file resolution rejects absolute paths, non-normal components, symlinks, non-files, and root escape; metric provenance binds actual content hashes rather than unverified declarations; `VALID_WITH_LIMITATIONS` preserves explicit unavailable counters, `INDETERMINATE` preserves incomplete track facts, and neither verdict adjudicates a scientific claim
- Verification summary: the committed seven-case corpus produced exact verdict/reason arrays for `VALID`, `MIGRATABLE`, `FORWARD_READABLE`, component-corrupt `INVALID`, ambiguous-track `INDETERMINATE`, unavailable-counter `VALID_WITH_LIMITATIONS`, and tampered-hash `INVALID`; repeated v0 migration was byte-identical and left the source unchanged; forward fixtures were hash-checked but not semantically interpreted; the CLI emitted one schema-valid JSON report and exact 0/2 exit semantics; the first retained gate stopped on a strict Clippy ownership finding and the corrected retained full gate passed formatting, strict workspace Clippy, 36 Rust tests, 2 Python tests, schema compatibility, docs, ADR, license, governance-ledger, terminology, and CI-topology checks on Windows x86_64
- Evidence paths and SHA-256 hashes: strict-lint failure record `BC-12-1784473567251415900` with stderr `d997c4616ab4a34784f2a7d7e625150b0c4fa8819bd003616a69432eb744b7e8`; passing machine record `BC-12-1784473635637114200` with stdout `8491f96ae6af4d609c018149cf5f12119a471d98c158fc3ab545dacde4f3a3c3` and stderr `629ea33644568a9227c3d08256534bee21f777241140196153effc59a1f470b2`; fixture matrix `6B55C615BE43BD4AA5B67B3AD83479C573CF923450ABDCC2831C797D51BAB2B6`; implementation `FA572946D8A380C31AEF1ED25D362F1079D5AF537FA415EA20C7B29099482959`; manifest schema `AAFEC1983B6DD1538C4FD02DF0E74B0248E9117213F6F18835A75EB18B749A5E`; report schema `AA6C9F8A973A58EC1C4710860D3365FFC64EE01DB6FB751060EF56E09984ACC8`; verifier `86D52211F9B077D39A77A6967D970D14A196611ADF23D0CB59DE02A93FCE0659`; `Cargo.lock` `2C9BA13AF614DB6DF5782166151E2CE0E85D97F72579998322023CB7EBA4B67C`
- Commit SHA: pending; report the focused commit and hosted run directly because the authorized prompt set ends here under the self-hash convention
- Risks/blockers/parked scope changed: R-15 remains active for later offline restoration and additional epochs and is controlled here by explicit current/old/future behavior plus frozen fixtures; R-16 remains controlled by the continuing publication addendum and no-slop gate; no parked scope revived and no claim adjudication, physical-host, instrument-completion, or C0–C5 claim is made
- Next eligible prompt: BR-01; it is outside this session's named BC-11/BC-12 authorization and was not started

### BC-12 hosted portability correction and closeout

- Initial implementation commit SHA: `2cc885aad3a1ce153e7afc557224a42b34f79f6e`
- Initial hosted verification: GitHub Actions run 29692334806, attempt 1, passed Linux x86_64 and macOS arm64/Intel but failed all three BC-12 corpus tests on Windows x86_64 because Git checkout converted referenced JSON fixture bytes from LF to CRLF while manifests bind exact stored-byte SHA-256 identities
- Correction: `.gitattributes` now freezes every repository JSON file as text with LF checkout bytes; a separate `core.autocrlf=true` checkout-index reproduction retained the three manifest-bound fixture hashes exactly and produced all seven expected CLI verdicts and exit codes
- Correction commit SHA and hosted rerun: `c8d03249920aa6ed98b353d9f046d84ddf8f3d66`; GitHub Actions run 29692636701, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Final status: passed and published; `origin/main` equals the correction commit, closing BC-12 and making BR-01 eligible

## 2026-07-19 — BR-01 — Adapter protocol and capability handshake

- Status: passed; closeout entry pending focused commit identity
- Authorization scope: user instruction `Commence STS of BR-01–BR-06 with my full authorization for this entire session.` plus the existing later-gated public source publication addendum
- Dependencies and source revision: BC-02 through BC-07 published; BC-12 correction hosted-green at `c8d03249920aa6ed98b353d9f046d84ddf8f3d66`
- Objective and exclusions: define the schema-epoch-1 child-process adapter operations, capability/version negotiation, deterministic seeds, monotonic deadlines, ordered state machine, replay/privilege facts, and exact failure posture; no network transport, algorithm API, process implementation, observer replay route, or hostile-code sandbox claim
- Reuse classification: extend the existing `bonsai-contracts`, generated Protobuf descriptor, BC-02 event, BC-05 track-fact, BC-06 work-class, BC-07 lineage, SHA-256, and ADR 0002 seams; introduce no production dependency and no parallel contract authority
- Files changed: adapter Protobuf and generated-build input, Rust validation/state machine and conformance tests, committed exact-outcome matrix, adapter-protocol architecture specification, Protobuf index, BC-12 hosted closeout, PSPR/root status, DEVLOG, verification log, and retained machine evidence
- Decisions/addenda: inherited stdin/stdout remains the only transport seam; supervisor and adapter sequences are independent and advance only on valid frames; capability declarations cover reset/work/feedback/events plus replay, privilege, filesystem, and network facts and are frozen by a SHA-256 fingerprint; configuration binds the accepted fingerprint; run and reset seeds are explicit and zero remains valid; deadlines are strictly increasing supervisor-monotonic bounds, not cross-process clock comparisons; invalid frames do not advance state; stop is permitted from any live state and every post-stop frame fails closed; track eligibility remains derived rather than self-attested
- Verification summary: six state-machine tests passed for the complete declared operation path, explicit presence of replay/privilege flags, configure-before-start, incompatible version, changed capability fingerprint, and post-stop traffic; the final retained full gate passed formatting, strict workspace Clippy, 42 Rust tests, 2 Python tests, schema compatibility, docs, ADR, license, governance-ledger, terminology, and CI-topology checks on Windows x86_64
- Evidence paths and SHA-256 hashes: final passing machine record `BR-01-FINAL-1784477271044328200` with stdout `6442ad3f111469e453463b12998fcbab7eae0e4780bd885788096328e98302d7` and stderr `fb3f2ad36f6dd7e817e5a713bdee56cae8c2c931eea5fd492469d27e151a327e`; pre-tightening passing records remain retained as historical evidence; outcome matrix `E0CD82C0C1DD6C6927E62EE56555FA330B2C4D52A0E8E0321E95299B4AEF85FB`; adapter schema `0B01498A4D6DE375D03CE00A51FEA909F4D111F3EDB1D5BE5E31049385042D0D`; implementation `845CD12080AB7706DE8166599400E487CAFD1FA80ADB2569863CE9A3DC9510A9`; verifier `9485AB2A886618734052ED79C6208BE1A43AC5B6CD3D51290CE86980BE4F4C0F`; `Cargo.lock` `2C9BA13AF614DB6DF5782166151E2CE0E85D97F72579998322023CB7EBA4B67C`
- Commit SHA: pending; append in BR-02 under the self-hash convention
- Risks/blockers/parked scope changed: R-07 remains active for BR-06 and BR-09; R-08 remains active for BR-02/BR-03 process and ingestion containment; both are controlled here by explicit capability facts, fail-closed sequencing, immutable fingerprints, and the no-sandbox boundary; R-16 remains controlled by the continuing publication addendum and no-slop gate; no parked scope revived and no live process, physical-host, instrument-completion, or C0–C5 claim is made
- Next eligible prompt after gate and publication: BR-02

### BR-01 closeout note

- Focused implementation commit SHA: `f8bc73b158a3a407cd7e252c76cfbeddfcce2654`
- Hosted verification: GitHub Actions run 29694408721, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BR-02 because the BR-01 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-19 — BR-02 — Bounded framed process transport

- Status: passed; closeout entry pending focused commit identity and hosted cross-OS run
- Authorization scope: user instruction `Commence STS of BR-01–BR-06 with my full authorization for this entire session.` plus the existing later-gated public source publication addendum
- Dependencies and source revision: BR-01 published and hosted-green at `f8bc73b158a3a407cd7e252c76cfbeddfcce2654`
- Objective and exclusions: implement bounded little-endian framing and a cross-platform Rust child supervisor plus standard-library Python adapter runtime with piped protocol stdin/stdout, independently drained bounded stderr, bounded pending queue, receive/shutdown timeouts, fail-closed containment, and clean process reaping; no network transport, event-semantic validation, launch/filesystem isolation policy, descendant resource enforcement, or hostile-code sandbox claim
- Reuse classification: implement a new `bonsai-runtime` crate at the accepted ADR 0002 and BR-01 process seam; reuse Prost, standard OS pipes/processes/threads/channels, workspace Python/pytest, and existing verification/governance infrastructure; add no new external dependency and create no parallel protocol authority
- Files changed: workspace/runtime crate manifest and lock, Rust frame/supervisor implementation plus cross-language process tests, Python framing runtime and fixture/tests, committed outcome matrix, process-transport specification, README runtime index, BR-01 hosted closeout, PSPR/root status, DEVLOG, verification log, and retained machine evidence
- Decisions/addenda: each frame is a nonzero `u32` little-endian length plus exact payload under a 16 MiB hard ceiling; declared size is checked before allocation; partial header/payload, oversize, timeout, flood/backpressure, stream, decode, I/O, spawn, shutdown, and thread failures have distinct stable codes; stdout enters a fixed-capacity channel and queue overflow closes the reader with bounded failure evidence; stderr is continuously drained, retains only a configured prefix, records total/truncation, and never enters protocol parsing; every transport error contains, kills, and reaps the child; drop is fail-safe; BR-06 retains authority for sanitized arguments/environment/handles/filesystem layout
- Verification summary: real Rust-to-Python and Python-to-Rust inherited-pipe exchange, clean shutdown, exact partial/oversized/stalled containment, bounded stderr separation, and repeatable flood rejection passed; the retained full gate passed formatting, strict workspace Clippy, 47 Rust tests, 3 Python tests, schema compatibility, docs, ADR, license, governance-ledger, terminology, and CI-topology checks on Windows x86_64; cross-OS hosted closure remains attached to the focused commit
- Evidence paths and SHA-256 hashes: final passing machine record `BR-02-FINAL-1784478702679705900` with stdout `e12cad842afd2a79b29ec7c7f5ea25953e713fb1a275b2051561de47546b222a` and stderr `e32b61c6e14dcaf42ec4fa6e0ecf931a3bc59255fa812c4184df323c0b9a876d`; the earlier pre-final passing record remains retained as history; outcome matrix `ABF7571E9E955750A4530BCB72B1A9D56BF0BA2E78FA818D8A342FFB6EBEA75F`; Rust implementation `64D24C2E6E8EF4B565A22D93C794CD6ED964E1F68811E4A6EBC740BECA0DA43B`; Python implementation `9FC659B590C9A690F5D3BDC428C83A2F579B47760E8C967795499EF549FECAFF`; verifier `D955F1864AB9924DBADC0AF424C20F9F54E3271A9C0C10362C1EFFD8DFA67950`; `Cargo.lock` `86692A6AB0CA6837B8187AA5F07B73B3B252CA6913991763CBFA1213879B15EC`
- Commit SHA: pending; append in BR-03 under the self-hash convention
- Risks/blockers/parked scope changed: R-08 remains active for BR-03 ingest and later descendant/platform controls and is controlled here by pre-allocation size checks, fixed queue/log bounds, stable failure records, kill/reap behavior, and the explicit non-sandbox claim; R-07 remains active for BR-06/BR-09; R-16 remains controlled by the publication addendum and no-slop gate; no parked scope revived and no physical-host, adversarial sandbox, instrument-completion, or C0–C5 claim is made
- Next eligible prompt after gate and publication: BR-03

### BR-02 closeout note

- Focused implementation commit SHA: `380b9764ea4c9cf6c80a9fadd2cdea75eb98aa9e`
- Hosted verification: GitHub Actions run 29695193065, attempt 1, passed the real Rust↔Python pipe fixture plus the complete repository gate on Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BR-03 because the BR-02 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-19 — BR-03 — Event ingestion validation

- Status: passed; closeout entry pending focused commit identity and hosted run
- Authorization scope: user instruction `Commence STS of BR-01–BR-06 with my full authorization for this entire session.` plus the existing later-gated public source publication addendum
- Dependencies and source revision: BC-09 published; BR-02 published and hosted-green at `380b9764ea4c9cf6c80a9fadd2cdea75eb98aa9e`
- Objective and exclusions: validate encoded event bounds, the BC-02 event contract, run/source/event/schema authorization, payload hashes/sizes, causal-parent bounds, observer-arrival rate limits, and ingestion lifecycle before the existing immutable segment writer is touched; emit bounded observer rejection evidence; no event repair, total ordering, duplicate/late/concurrency classification, run recovery, or launch isolation
- Reuse classification: implement a new `bonsai-ingest` crate around the existing `bonsai-contracts::decode_and_validate_event` and `bonsai-bundle::SegmentWriter`; extend `EventValidationError` with stable codes; reuse Serde/JSON/SHA-256/tempfile already locked in the workspace; add no new external dependency and no second event/storage authority
- Files changed: workspace/ingest crate manifest and lock, event validation code export, ingestor/lifecycle/rate/rejection implementation and deterministic fuzz/property tests, committed outcome matrix, ingestion specification, BR-02 hosted closeout, PSPR/root status, DEVLOG, verification log, and retained machine evidence
- Decisions/addenda: the ingestor exclusively borrows its segment writer; validation order is lifecycle, envelope size, envelope contract/hash, run, source, event type/payload, parent count, schema epoch/minor, and observer-arrival rate; rate state advances only after append succeeds; source sequence/duplicate/late/missing-parent/concurrent/event-clock facts are preserved for BR-04; rejection JSON contains only stable code, byte count, and fixed-size decoded identities, is bounded by record count and total bytes, evicts oldest entries with a saturating dropped count, and retains no arbitrary adapter detail
- Verification summary: four focused tests passed, proving the committed exact-outcome matrix, one valid original-byte append, zero invalid appends, bounded rate/rejection handling, and 2,048 deterministic pseudo-random inputs without panic; the retained full gate passed formatting, strict workspace Clippy, 51 Rust tests, 3 Python tests, schema compatibility, docs, ADR, license, governance-ledger, terminology, and CI-topology checks on Windows x86_64
- Evidence paths and SHA-256 hashes: passing machine record `BR-03-1784479641202426700` with stdout `0dbeb1c40a69101ce6e86c9bc5f3d81348ee0ef5814ce719b5c88afe9b37d39c` and stderr `36aa3fe3031538547a8a4ccc8444f47c173d06ba1d89e15bbfe16918a5e8f040`; outcome matrix `7B290AF924E1334463C9B95F3D643439C8DE9B2C5E8913701F7A261A00BEFDEB`; implementation `0062049211879B23E1008AE78A708AF542C08618354B3C56F76262122501B408`; verifier `73DD6ADBD09CCB3AB8E95E4F664EF349896C1CC2480DD4B386E0185FEF74B484`; `Cargo.lock` `FF83EB84B00CA8FC4DE402433CA839A8DCDBBE6D817FB3A82832D4B3E1D8587E`
- Commit SHA: pending; append in BR-04 under the self-hash convention
- Risks/blockers/parked scope changed: R-08 is controlled at the ingest seam by envelope/payload/parent/rate bounds, validate-before-append, deterministic failure codes, and bounded rejection retention but remains active for later descendant/platform controls; R-09 remains active for observer-volume/reserve integration; R-07 remains active for BR-06/BR-09; R-16 remains controlled by the publication addendum and no-slop gate; no parked scope revived and no ordering, physical-host, instrument-completion, or C0–C5 claim is made
- Next eligible prompt after gate and publication: BR-04

### BR-03 closeout note

- Focused implementation commit SHA: `e406d26157fc36cff4b9c02e55eeb425ce93bfbc`
- Hosted verification: GitHub Actions run 29695677005, attempt 1, passed Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel at that exact commit
- Ledger rule: appended by BR-04 because the BR-03 commit could not contain its own immutable hash or post-push hosted-run identity

## 2026-07-19 — BR-04 — Event partial-order semantics

- Status: passed; closeout entry pending focused commit identity and hosted run
- Authorization scope: user instruction `Commence STS of BR-01–BR-06 with my full authorization for this entire session.` plus the existing later-gated public source publication addendum
- Dependencies and source revision: BR-03 published and hosted-green at `e406d26157fc36cff4b9c02e55eeb425ce93bfbc`
- Objective and exclusions: deterministically derive per-source and causal partial-order edges and classify late, duplicate, missing-parent, concurrent, clock-regression, sequence-conflict, sequence-gap, and cycle facts under explicit graph bounds; no wall-time total order, synthetic parent, ambiguity repair, event rejection, lifecycle recovery, or scientific causality inference
- Reuse classification: extend `bonsai-ingest` at the post-validation seam using the existing BC-02 envelope identities/sequences/parents/times and standard `BTreeMap`/`BTreeSet` graph structures; add no dependency, storage authority, or alternate event contract
- Files changed: ordering engine/types and deterministic integration fixtures/tests, committed outcome matrix, ordering specification, BR-03 hosted closeout, PSPR/root status, DEVLOG, verification log, and retained machine evidence
- Decisions/addenda: caller-supplied immutable arrival indices preserve arrival-relative late semantics while making collection iteration order irrelevant; unique contiguous source sequences and present causal parents are the only edge authorities; conflicts/gaps/missing parents/cycles remain explicit instead of being repaired; concurrency means no reachability in either direction and is unaffected by shared ancestors, arrival proximity, Unix wall time, or identity order; all report collections are identity-sorted and resource bounds apply before graph construction
- Verification summary: four focused tests passed for the committed matrix, exact class/edge/concurrency results, conflict/gap/cycle refusal to fabricate edges, wall-time non-authority, and exact report equality across every rotate/reverse collection permutation with preserved arrival indices; the retained full gate passed formatting, strict workspace Clippy, 55 Rust tests, 3 Python tests, schema compatibility, docs, ADR, license, governance-ledger, terminology, and CI-topology checks on Windows x86_64
- Evidence paths and SHA-256 hashes: passing machine record `BR-04-1784480488552125000` with stdout `534afbcea4b6c1fc2c2d64b06d47de6dddb389093fc19b30f0f8f289aedde785` and stderr `81d5e8987a260c7755cbaa481df2f40347b3d2dad7f95d3226886b7faa63bf55`; outcome matrix `065C59C29629C6CEB9F95378FEA5259B17DB67ECA7ADADB8E756B0FF7A2D3DA4`; implementation `B4AB1F3C5BCD5B2A6148EB1F9979AC51F76AA52005882B5AF16F7DE6E653A94A`; verifier `F0889F66D31EA0987A099F036DDB235CC0A0F31474ABF6322B227EE8623A1ED6`; `Cargo.lock` `FF83EB84B00CA8FC4DE402433CA839A8DCDBBE6D817FB3A82832D4B3E1D8587E`
- Commit SHA: pending; append in BR-05 under the self-hash convention
- Risks/blockers/parked scope changed: R-10 remains active for later numeric/semantic cross-platform equivalence and is controlled here by integer/identity-only sorted semantics plus hosted exact equality; R-07/R-08/R-09 remain unchanged for isolation, controls, and observer-volume integration; R-16 remains controlled by the publication addendum and no-slop gate; no parked scope revived and no total-order, scientific-causality, instrument-completion, or C0–C5 claim is made
- Next eligible prompt after gate and publication: BR-05
