# BONSAI verification log

This is an append-only human index. Machine records created by `cargo xtask verify` use `bonsai.verification-record/v1`; raw command outputs are retained only when the record names their relative path and SHA-256. Bootstrap entries before BG-06 honestly state that no output artifact file existed.

## VER-BG01-BOOTSTRAP — BG-01 — 2026-07-19T00:58:55Z

- Source revision and dirty state: `7d0ab846e46a9f38c3bd017da4837bf254b76bdc`; dirty because root README was the prompt change
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: not applicable
- Command: PowerShell assertion set using `git -c safe.directory=<authoritative-root> rev-parse --show-toplevel`, session `git rev-parse`, `git rev-parse HEAD`, `git rev-parse origin/main`, `git ls-tree`, and indexed-path rejection
- Start/end/duration: `2026-07-19T00:58:55.0820621Z` / `2026-07-19T00:58:55.6442674Z` / 0.5622053 s
- Exit code: 0
- Stdout/stderr artifact hashes: not retained; BG-06 did not yet exist
- Fixtures/manifests/bundle IDs: baseline six-document index
- Counter availability and privileges: not applicable; no privileged action
- Result: pass
- Reviewer/attestation: automated assertions plus STS reconciliation

## VER-BG02-BOOTSTRAP — BG-02 — 2026-07-19T01:01:18Z

- Source revision and dirty state: `210024faf5a315a1381318a408b49b6ae48fd751`; dirty with BG-02 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Python 3.14.4; no dependency lock required by the bootstrap checker
- Command: `py -3 scripts/check_docs.py`
- Start/end/duration: `2026-07-19T01:01:18.3375911Z` / `2026-07-19T01:01:18.5789239Z` / 0.2413328 s
- Exit code: 0
- Stdout/stderr artifact hashes: not retained; BG-06 did not yet exist
- Fixtures/manifests/bundle IDs: eight Markdown files
- Counter availability and privileges: not applicable
- Result: pass
- Reviewer/attestation: automated local-link and STS-warning checker

## VER-BG03-BOOTSTRAP — BG-03 — 2026-07-19T01:05:28Z

- Source revision and dirty state: `7193f224aa00ab2cbafeec0809ac618ea93dce6f`; dirty with BG-03 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Python 3.14.4
- Command: `py -3 -B scripts/check_adrs.py`; `py -3 -B scripts/check_docs.py`
- Start/end/duration: `2026-07-19T01:05:28.9905891Z` / `2026-07-19T01:05:29.3530384Z` / 0.3624493 s
- Exit code: 0 / 0
- Stdout/stderr artifact hashes: not retained; BG-06 did not yet exist
- Fixtures/manifests/bundle IDs: D-01 through D-21 and seven ADRs
- Counter availability and privileges: not applicable
- Result: pass
- Reviewer/attestation: exact index/body coverage checker

## VER-BG04-BOOTSTRAP — BG-04 — 2026-07-19T01:08:48Z

- Source revision and dirty state: `51b95630399816e5428e8effa6ef7fc6870f7a6c`; dirty with BG-04 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Python 3.14.4
- Command: `py -3 -B scripts/check_license.py`; `py -3 -B scripts/check_docs.py`
- Start/end/duration: `2026-07-19T01:08:48.8138523Z` / `2026-07-19T01:08:49.0867750Z` / 0.2729227 s
- Exit code: 0 / 0
- Stdout/stderr artifact hashes: not retained; BG-06 did not yet exist
- Fixtures/manifests/bundle IDs: `LICENSE-APACHE`, `LICENSE-MIT`, publication policy
- Counter availability and privileges: not applicable
- Result: pass
- Reviewer/attestation: automated license/policy and docs checkers

## VER-BG05-CLEAN — BG-05 — 2026-07-19T01:17:56Z

- Source revision and dirty state: `444eb6b446d2adf0d7ff34104ca6fb373cbbea2e`; clean before and after
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; uv 0.11.29; Python 3.14.4; `Cargo.lock` SHA-256 `84C410D11522EEC3BCBC822EC9C6B15B987F35B91402597886050B01FAA2F17B`; `uv.lock` SHA-256 `3C745B23FDB0DF09F26CD6652FF1207BA9C5FC7577955B67C594BEA7468E37AC`
- Command: `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-features`; `uv run --frozen ruff check .`; `uv run --frozen pyright`; `uv run --frozen pytest`
- Start/end/duration: `2026-07-19T01:17:56.5768246Z` / `2026-07-19T01:18:01.7420149Z` / 5.1651903 s
- Exit code: 0 for every command
- Stdout/stderr artifact hashes: not retained; BG-06 did not yet exist
- Fixtures/manifests/bundle IDs: one Rust scaffold test; one Python scaffold test
- Counter availability and privileges: not applicable; temporary dependency installation/build permission only
- Result: pass
- Reviewer/attestation: complete clean-checkout M0 universal gate

## VER-BG06-GATE — BG-06 — 2026-07-19T01:30:44Z

- Source revision and dirty state: `840bd1382c942d97ec4b6b1c82240e4e5bd970e6`; dirty with BG-06 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; uv 0.11.29; Python 3.14.4; `Cargo.lock` SHA-256 `EC11BBBD4C08490299A1AD648A86D7D0B9115B4EBE0B3310038469DFFC36EBEB`; `uv.lock` SHA-256 `3C745B23FDB0DF09F26CD6652FF1207BA9C5FC7577955B67C594BEA7468E37AC`
- Command: universal Cargo and uv gates; `py -3 -B scripts/check_docs.py`; `py -3 -B scripts/check_adrs.py`; `py -3 -B scripts/check_license.py`
- Start/end/duration: `2026-07-19T01:30:44.9085757Z` / `2026-07-19T01:30:52.9357285Z` / 8.0271528 s
- Exit code: 0 for every gate command
- Stdout/stderr artifact hashes: manual pass stdout SHA-256 `b6ef6807dd96d18b833474ad68e7a23a29e562a29c67afa71a59fb9a73df0068`; empty streams SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`; manual fail stderr SHA-256 `02d3485f24dab97508da47c674806f5ac2d27a6174433a22545a169a20e80d73`
- Fixtures/manifests/bundle IDs: `verification_fixture::passing_and_failing_commands_produce_distinct_sanitized_records`
- Counter availability and privileges: not applicable; no environment enumeration
- Result: pass
- Reviewer/attestation: automated universal gates, integration fixture, and manual machine-record inspection

## VER-BG07-GATE — BG-07 — 2026-07-19T01:34:54Z

- Source revision and dirty state: `369bad35ee1c7599569c3e6fb12fceab5332e7ab`; dirty with BG-07 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: unchanged from VER-BG06-GATE
- Command: universal Cargo and uv gates; docs, ADR, license, and governance-ledger checkers
- Start/end/duration: `2026-07-19T01:34:54.8393819Z` / `2026-07-19T01:35:02.8639980Z` / 8.0246161 s
- Exit code: 0 for every gate command
- Stdout/stderr artifact hashes: command output retained in the STS session transcript; ledger files are committed source artifacts
- Fixtures/manifests/bundle IDs: negative missing-owner and missing-revival fixtures; R-01–R-16; P-01–P-09
- Counter availability and privileges: not applicable
- Result: pass
- Reviewer/attestation: automated schema/coverage checks and universal gates

## VER-BG08-GATE — BG-08 — 2026-07-19T01:39:52Z

- Source revision and dirty state: `98ed62cd393f9c4cf6927ec8ce0efaa85a732c3a`; dirty with BG-08 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: unchanged from VER-BG06-GATE
- Command: universal Cargo and uv gates; docs, ADR, license, governance-ledger, and terminology checkers
- Start/end/duration: universal `2026-07-19T01:39:52.1909184Z` / `2026-07-19T01:40:01.4419485Z` / 9.2510301 s; governance `2026-07-19T01:40:10.9836642Z` / `2026-07-19T01:40:11.9531874Z` / 0.9695232 s
- Exit code: 0 for every gate command
- Stdout/stderr artifact hashes: command output retained in the STS session transcript; registry is a committed source artifact
- Fixtures/manifests/bundle IDs: duplicate-term and unitless-numeric negative fixtures; terminology registry epoch 1
- Counter availability and privileges: not applicable
- Result: pass
- Reviewer/attestation: automated registry/schema, documentation, and universal gates

## VER-BG09-LOCAL — BG-09 — 2026-07-19T01:44:57Z

- Source revision and dirty state: `85e408def2e4e74ef472aa46d29ce4d44f8b677d`; dirty with BG-09 workflow changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: unchanged from VER-BG06-GATE; workflow pins Rust 1.96.0, Python 3.12, uv 0.11.29, and full action commit SHAs
- Command: universal Cargo and uv gates; all governance checkers including `scripts/check_ci.py`; sanitized CI-evidence writer fixture
- Start/end/duration: `2026-07-19T01:44:57.0437045Z` / `2026-07-19T01:45:02.5130270Z` / 5.4693225 s
- Exit code: 0 for every local gate command
- Stdout/stderr artifact hashes: CI-evidence fixture SHA-256 `28f0760b2ee440ded58d4b783c3a6429bd7c213d9c90832fef593a92fa7ad2f8`
- Fixtures/manifests/bundle IDs: missing-Windows-runner negative fixture; hosted classification fixture
- Counter availability and privileges: hosted runner evidence not yet available; no physical counters claimed
- Result: pass locally; live hosted gate pending
- Reviewer/attestation: automated topology, evidence-boundary, and universal gates

## VER-BG09-LIVE — BG-09 — 2026-07-19T01:47:34Z

- Source revision and dirty state: `59e474a8a3eeddbc071b02c0152d8d7925b9af27`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, uv 0.11.29; source lockfiles at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29669146969, attempt 1
- Start/end/duration: run `2026-07-19T01:47:34Z` through `2026-07-19T01:48:56Z`; per-job timestamps retained by GitHub
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: Linux `7139746343B454756FD8F293ACD77BAC453F9DC6FA13DBA71BAD8BB2E9BA1F88`; macOS arm64 `EFC562205BE6FFB843B38EECD909E6F673AFD595F1DEEDEC33EB813064601FFF`; macOS x86_64 `A65D4768622F798061BB0DEB61BC4201A45C14A03907FCAE133BCE72BD4AB97B`; Windows `5D31F7F1C40216C26F0FC07B3561FDA302AA6D3549F875630D94F872C4E8E909`
- Fixtures/manifests/bundle IDs: artifact names `baseline-linux-x86_64`, `baseline-macos-arm64`, `baseline-macos-x86_64`, `baseline-windows-x86_64`
- Counter availability and privileges: no physical counter evidence; artifacts set `physical_acceptance=false`, `energy_claim=false`, and `long_duration_claim=false`
- Result: pass
- Reviewer/attestation: GitHub job conclusions plus downloaded artifact content/hash inspection

## VER-BG10-PRECOMMIT — BG-10 — 2026-07-19T01:54:31Z

- Source revision and dirty state: `d2774154931f91a3205ac415aed7c791cddd5035`; dirty with BG-10 checkpoint changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: unchanged from VER-BG06-GATE
- Command: universal Cargo and uv gates; every component governance checker; `py -3 -B scripts/check_m0.py --allow-dirty`
- Start/end/duration: `2026-07-19T01:54:31.9333946Z` / `2026-07-19T01:54:42.0358375Z` / 10.1024429 s
- Exit code: 0 for every gate command
- Stdout/stderr artifact hashes: command output retained in the STS session transcript; M0 checkpoint is a committed source artifact after the next commit
- Fixtures/manifests/bundle IDs: M0 required-file and prompt-status audit
- Counter availability and privileges: no physical counter or capability evidence; none claimed
- Result: pass pre-commit; clean checkpoint pending
- Reviewer/attestation: automated universal and source-of-truth audit

## VER-BG10-CLEAN — BG-10 — 2026-07-19T01:55:52Z

- Source revision and dirty state: `1b68656057a6920f5a087e03d1ca181f914b2791`; clean before and after
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: unchanged from VER-BG06-GATE
- Command: clean-tree assertion; universal Cargo/uv gates; all governance and M0 checkers; `gitleaks git . --log-opts="--all" --redact`
- Start/end/duration: `2026-07-19T01:55:52.0460274Z` / `2026-07-19T01:56:01.5553664Z` / 9.5093390 s
- Exit code: 0 for every gate and scan command
- Stdout/stderr artifact hashes: command output retained in the STS session transcript; Gitleaks scanned 13 commits / approximately 304.63 KB and reported no leaks
- Fixtures/manifests/bundle IDs: complete M0 required-file/status/claim/scope audit
- Counter availability and privileges: no physical counter or capability evidence; none claimed
- Result: pass
- Reviewer/attestation: clean automated source-of-truth gate and all-history secret scan

## VER-BC01-ENV-FAIL — BC-01 — 2026-07-19T02:15:33Z

- Source revision and dirty state: `8873e13444512a5035f45527c6cacff5d14301e5`; dirty with BC-01 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; nested PowerShell could not resolve uv; `Cargo.lock` SHA-256 `EC11BBBD4C08490299A1AD648A86D7D0B9115B4EBE0B3310038469DFFC36EBEB`; `uv.lock` SHA-256 `3C745B23FDB0DF09F26CD6652FF1207BA9C5FC7577955B67C594BEA7468E37AC`
- Command: machine record `BC-01-1784427333002204600`; universal gate through `cargo xtask verify`
- Start/end/duration: `2026-07-19T02:15:33.002Z` / `2026-07-19T02:15:36.122Z` / 3.1200654 s
- Exit code: 1
- Stdout/stderr artifact hashes: stdout `3f45cd4e4574305d3a60acfa79972da4068f9c76738cb1cc0fb8fdb6512fc971`; stderr `37ca0d2f0f3b5df209dda904c173e01dec21a78ab76c9ad312a8621d5c0e495e`
- Fixtures/manifests/bundle IDs: additive and four rejection schema fixtures passed before the environment failure
- Counter availability and privileges: not applicable; no physical or privileged claim
- Result: fail; environment/tool-path failure, not a schema or product failure
- Reviewer/attestation: machine verification record retained with exact command and output

## VER-BC01-GATE — BC-01 — 2026-07-19T02:19:00Z

- Source revision and dirty state: `8873e13444512a5035f45527c6cacff5d14301e5`; dirty with BC-01 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; checksum-verified official uv 0.11.29; `Cargo.lock` SHA-256 `EC11BBBD4C08490299A1AD648A86D7D0B9115B4EBE0B3310038469DFFC36EBEB`; `uv.lock` SHA-256 `3C745B23FDB0DF09F26CD6652FF1207BA9C5FC7577955B67C594BEA7468E37AC`
- Command: machine record `BC-01-1784427540937208400`; `cargo fmt --all --check`; workspace Clippy/tests; frozen Ruff/Pyright/Pytest; `cargo xtask schema-check`; docs, ADR, license, governance-ledger, terminology, and CI-topology checks
- Start/end/duration: `2026-07-19T02:19:00.937Z` / `2026-07-19T02:19:24.728Z` / 23.7921108 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `4b08b7548416b12819e7796578ca19101d65ab4183f4947a51912373e85a4462`; stderr `d90189be1ef824c4bf3053e76efa053cf5fd78b771b0a73db70bacf28fae7020`
- Fixtures/manifests/bundle IDs: `baseline.json`; additive digest `c60bb5796a2155ae8a4d927774b4628d272b8ccf48488dcfc552d426eb4bda6e`; field-renumbering `FIELD_RENUMBERED`; field-reuse `FIELD_REUSE`; silent-unit-change `UNIT_CHANGED`; unversioned JSON `JSON_VERSION_MISSING`
- Counter availability and privileges: not applicable; no physical, energy, enforcement, or integration claim
- Result: pass
- Reviewer/attestation: machine verification record plus exact frozen compatibility outcomes

## VER-BC01-HOSTED — BC-01 — 2026-07-19T02:25:23Z

- Source revision and dirty state: `a0b4aba07191d8035330bec4f0eeb0bf64bb31e8`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29670167856, attempt 1
- Start/end/duration: `2026-07-19T02:25:23Z` / `2026-07-19T02:26:38Z` / 75 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88147590507, 88147590514, 88147590517, and 88147590520
- Fixtures/manifests/bundle IDs: hosted classification artifacts generated per matrix job
- Counter availability and privileges: no physical counter evidence; workflow classification denies physical, energy, and long-duration claims
- Result: pass
- Reviewer/attestation: GitHub job conclusions inspected for every matrix entry

## VER-BC02-WINDOWS-LOCK — BC-02 — 2026-07-19T02:35:11Z

- Source revision and dirty state: `a0b4aba07191d8035330bec4f0eeb0bf64bb31e8`; dirty with BC-02 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; final BC-02 locks were present
- Command: machine records `BC-02-1784428511706744200` and `BC-02-1784428577885004300`; full gate invoked by the in-tree verifier
- Start/end/duration: first `2026-07-19T02:35:11.706Z` / `2026-07-19T02:35:23.197Z`; retry `2026-07-19T02:36:17.885Z` / `2026-07-19T02:36:19.936Z`
- Exit code: 101 for both
- Stdout/stderr artifact hashes: first stdout `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`, stderr `3b3fd939dc66290d6ec62000fbbbf6242754b6725924415e6cbbad6ab961cb08`; retry stdout identical, stderr `9be1a4ca8cf6a84ffa824411891a896a090d047fdecefcfa4e8ef66d08647580`
- Fixtures/manifests/bundle IDs: no product fixture failure; focused event tests had passed before these records
- Counter availability and privileges: not applicable
- Result: fail; Windows denied replacement of the running `target/debug/bonsai-xtask.exe` when the workspace integration test rebuilt it
- Reviewer/attestation: exact Cargo error retained; gate rerun unchanged using a temporary copy outside Cargo target

## VER-BC02-GATE — BC-02 — 2026-07-19T02:37:06Z

- Source revision and dirty state: `a0b4aba07191d8035330bec4f0eeb0bf64bb31e8`; dirty with BC-02 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `0A84D63445D25C6CBCA6FE74AA68EA656CB35EE13A09B576A41A568A7E644268`; `uv.lock` SHA-256 `EC18CF61A7A382BECA7F65105B6E427F3FF70F37CF99910E2E63902D7C900E43`
- Command: machine record `BC-02-1784428626815212000`; unchanged universal/governance gate run through a byte-identical temporary copy of the built repository verifier
- Start/end/duration: `2026-07-19T02:37:06.815Z` / `2026-07-19T02:37:16.087Z` / 9.2723955 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `1a318078e6b2ff263443932379c4d30f16d4448d4101b816b38271503131a081`; stderr `764edc6682fe3a2cc67bdbedbc6c3353e3ec768ef2dfb7892df148c4551f3c9b`
- Fixtures/manifests/bundle IDs: event envelope schema SHA-256 `A515C37F366EE16C58DC82608493F58FDFE6C66E251F384318EB40E610B8FAA1`; Python/Rust unknown-field 99 relay fixture; Rust invalid-ID/time/hash fixtures
- Counter availability and privileges: no physical, clock-synchronization, or global-order claim
- Result: pass
- Reviewer/attestation: full automated gate plus exact binary-preservation assertion
