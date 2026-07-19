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

## VER-BC02-HOSTED — BC-02 — 2026-07-19T02:41:34Z

- Source revision and dirty state: `127b20b68957fb1473fba670fe4cd411187c062e`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29670584785, attempt 1
- Start/end/duration: `2026-07-19T02:41:34Z` / `2026-07-19T02:43:44Z` / 130 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88148698568 (macOS Intel), 88148698569 (Windows), 88148698584 (Linux), and 88148698595 (macOS arm64)
- Fixtures/manifests/bundle IDs: hosted classification artifacts generated per matrix job; event-envelope and cross-language conformance gates ran from the recorded source revision
- Counter availability and privileges: no physical counter evidence; workflow classification denies physical, energy, and long-duration claims
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and every matrix-job conclusion inspected through the GitHub API

## VER-BC06-GATE — BC-06 — 2026-07-19T04:18:56Z

- Source revision and dirty state: `eaa0e52ec5a6dc78ab1a360f2a11c2201c7a5e9d`; dirty with BC-06 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `BA7C806E67A42D80EC8D0D0D9781F937BEDA17B2B2B291C333BFDE5FFE9ABA04`
- Command: final machine record `BC-06-1784434736275348400`; full universal/schema/governance gate with uv cache redirected to ignored repository `target` storage
- Start/end/duration: `2026-07-19T04:18:56.275348400Z` / `2026-07-19T04:19:07.475306200Z` / 11.2002545 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `fd7fb2cbddb47a2b64f862baf0665b1b9b62e63182e80a6ccf638b0cb9fd9a45`; stderr `7c992754329533ffa7fd6c392ea75036966df312bbcf39be27e6a8158698d8b2`
- Fixtures/manifests/bundle IDs: policy canonical SHA-256 `5053b8c5b78e46d1bf45b542815598f5fd127981ed61f1311938879badc77b49`; policy schema canonical SHA-256 `d2bc586d01c69ee7f1202ef3d8f324692661b6ecc8266c514ba0f25b2f32e877`; four scopes, nine work classes, five decision outcomes
- Counter availability and privileges: measured/estimated states are fixture evidence; no live counters, privileges, resource arithmetic, scheduling, or backend enforcement
- Result: pass
- Reviewer/attestation: exact Protobuf round-trip reconstruction inputs, unavailable-without-zero and strict outcome-action validation, resource-policy semantic validation, and full automated local gate; external verifier SHA-256 `87A82D2A9C0C663BF63FF634C118AAAB2DA7AA17D322A66B496707A1CC4CF733` matched in-tree before execution

## VER-BC06-ENV-FAIL — BC-06 — 2026-07-19T04:12:56Z

- Source revision and dirty state: `eaa0e52ec5a6dc78ab1a360f2a11c2201c7a5e9d`; dirty with BC-06 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local sandbox; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; uv 0.11.29; source locks unchanged
- Command: machine record `BC-06-1784434376427187100`; initial full universal/schema/governance gate
- Start/end/duration: `2026-07-19T04:12:56.427187100Z` / `2026-07-19T04:13:05.586119100Z` / 9.1591596 s
- Exit code: 2
- Stdout/stderr artifact hashes: stdout `291e8d4ca01a074e8084c6806d88d67763a776aff9712fc9dc4e93e79edb3a32`; stderr `4c1d1214c7d267421865593424cd942da3f6dca51514f771e8e50d134cef117d`
- Fixtures/manifests/bundle IDs: all 12 Rust contract tests passed before the Python stage
- Counter availability and privileges: not applicable; uv was denied access to `AppData/Local/uv/cache` by the workspace sandbox
- Result: fail, environment-only
- Reviewer/attestation: rerun retained identical gates and uv version with cache redirected to ignored repository `target` storage; no product or verification requirement changed

## VER-BC05-GATE — BC-05 — 2026-07-19T03:41:06Z

- Source revision and dirty state: `a694e2380b907d04aea41bca321bb091f6c2ba28`; dirty with BC-05 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `BA7C806E67A42D80EC8D0D0D9781F937BEDA17B2B2B291C333BFDE5FFE9ABA04`
- Command: machine record `BC-05-1784432466868264500`; full universal/schema/governance gate
- Start/end/duration: `2026-07-19T03:41:06.8682645Z` / `2026-07-19T03:41:18.3836683Z` / 11.5157002 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `f64546cc170ae1c2139942911b5e5b061f94d209e201d41179dd64e3e7b892b0`; stderr `9f28fbdc232e471eed34de59792aff947d843343f19bc7bd794a02b145c39f49`
- Fixtures/manifests/bundle IDs: seven track cases; schema canonical SHA-256 `eefeeba41b7a875c02bb6f5104ad6f02f2d3c16582594c57c4e669c798e6f2fa`
- Counter availability and privileges: runtime facts are fixtures only; no live isolation, privileged input, replay, or enforcement claim
- Result: pass
- Reviewer/attestation: exact derived outcomes/reason codes and full automated local gate; external verifier SHA-256 `27F9E6B316D53BA2C379C7CC96176CE670B99440D173D0D7B6BF28FF034C8FC3` matched in-tree

## VER-BC05-HOSTED — BC-05 — 2026-07-19T03:45:52Z

- Source revision and dirty state: `eaa0e52ec5a6dc78ab1a360f2a11c2201c7a5e9d`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29672261289, attempt 1
- Start/end/duration: `2026-07-19T03:45:52Z` / `2026-07-19T03:48:18Z` / 146 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88153321366 (Linux), 88153321358 (macOS Intel), 88153321357 (Windows), and 88153321363 (macOS arm64)
- Fixtures/manifests/bundle IDs: hosted schema gate exercised all seven track-classification fixtures on every matrix job
- Counter availability and privileges: fixture-only contract evidence; no live isolation or privileged-input evidence; workflow classification denies physical, energy, and long-duration claims
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and every matrix-job conclusion inspected through the GitHub API

## VER-BC04-GATE — BC-04 — 2026-07-19T03:28:30Z

- Source revision and dirty state: `d31e4a6e8126697357e7f0870f434ee24881e664`; dirty with BC-04 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; checksum-verified official uv 0.11.29; `Cargo.lock` SHA-256 `BA7C806E67A42D80EC8D0D0D9781F937BEDA17B2B2B291C333BFDE5FFE9ABA04`; `uv.lock` SHA-256 `EC18CF61A7A382BECA7F65105B6E427F3FF70F37CF99910E2E63902D7C900E43`
- Command: machine record `BC-04-1784431710849423900`; `cargo fmt --all --check`; workspace Clippy/tests; frozen Ruff/Pyright/Pytest; `cargo xtask schema-check`; docs, ADR, license, governance-ledger, terminology, and CI-topology checks
- Start/end/duration: `2026-07-19T03:28:30.8494239Z` / `2026-07-19T03:28:41.8276316Z` / 10.9784885 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `6236464b9c190f686ab4bda19163caa493fc2f222dd5b69fe34dc94c4aac09f1`; stderr `2d341ac1bc78080ced2b174604a62a4fe54707bf6599f1c81a856d6f9d9763b7`
- Fixtures/manifests/bundle IDs: `raw-sensitive.json`; `sanitized-expected.json`; sanitized canonical SHA-256 `0bb0b95eaa8d0440c417a316b09a3694658036cd48592a8fc21ef7b8ac975514`; schema canonical SHA-256 `f1fd3f59feab9ebbf6c06581d0011371ba8e4a7d68eb10e202bdbe4ad55830b5`
- Counter availability and privileges: collector interfaces and states only; no live host enumeration, physical counter, privileged collector, calibration, enforcement, or energy evidence
- Result: pass
- Reviewer/attestation: strict raw/sanitized fixture equality, forbidden-value absence, retained reproducibility-field assertions, Rust contract decoding, Draft 2020-12 validation, and full automated local gate; external verifier SHA-256 `F81144D316A5EB77F682E13535F20C0DCF53AE723171C82C7FD53FFCC6FB7AEF` matched the in-tree binary before execution

## VER-BC04-HOSTED — BC-04 — 2026-07-19T03:32:47Z

- Source revision and dirty state: `a694e2380b907d04aea41bca321bb091f6c2ba28`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29671931286, attempt 1
- Start/end/duration: `2026-07-19T03:32:47Z` / `2026-07-19T03:35:11Z` / 144 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88152460987 (Linux), 88152461016 (macOS Intel), 88152461020 (Windows), and 88152461039 (macOS arm64)
- Fixtures/manifests/bundle IDs: hosted schema gate exercised the raw-sensitive and exact sanitized platform-inventory fixtures on every matrix job
- Counter availability and privileges: no physical counter evidence; workflow classification denies physical, energy, and long-duration claims
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and every matrix-job conclusion inspected through the GitHub API

## VER-BC03-GATE — BC-03 — 2026-07-19T03:05:32Z

- Source revision and dirty state: `127b20b68957fb1473fba670fe4cd411187c062e`; dirty with BC-03 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; checksum-verified official uv 0.11.29; `Cargo.lock` SHA-256 `F2565497C1C59EBB1C22F88FCA096A0D05E1EFD9435F99D46C71E4DCFDF17D22`; `uv.lock` SHA-256 `EC18CF61A7A382BECA7F65105B6E427F3FF70F37CF99910E2E63902D7C900E43`
- Command: machine records `BC-03-1784430332012813700` and `BC-03-1784430432949968400`; each ran `cargo fmt --all --check`, workspace Clippy/tests, frozen Ruff/Pyright/Pytest, `cargo xtask schema-check`, and docs/ADR/license/governance-ledger/terminology/CI-topology checks
- Start/end/duration: initial `2026-07-19T03:05:32.0128137Z` / `2026-07-19T03:06:43.2602239Z` / 71.2493667 s; confirmation `2026-07-19T03:07:12.9499684Z` / `2026-07-19T03:07:29.4159038Z` / 16.4663853 s
- Exit code: 0 for both recorded invocations
- Stdout/stderr artifact hashes: initial stdout `44068a1b16625c008f56bf0793a650c797fa30781bbcfef9cba31156a8004518`, stderr `f3f69f47d929d3395e12716ade68822210a2e922c6cf58ee723ecd0664ccfc0a`; confirmation stdout `c217a429e089398bd0d7d4e02dd4993dbfa4e6fb283bb4b36b424620eabce721`, stderr `d07e8f3303112a7a301dfe1e290bcac2c67666a972a65ee56479fb9df68d599a`
- Fixtures/manifests/bundle IDs: valid manifest canonical SHA-256 `dc596b67136ae83046831e381cf0a5deab0719d54e874c5c26facc95ce140f57`; schema canonical SHA-256 `e4942f9d6a254cb31c574c8899b4d0814b6e421c38a0c9f889b1c1f61dd4a523`; `MANIFEST_REPLAY_REQUIRED`, `MANIFEST_RESOURCE_REQUIRED`, and `MANIFEST_SEEDS_REQUIRED` rejection fixtures
- Counter availability and privileges: manifest counter expectations are declarations only; no physical counter, resource-enforcement, energy-fidelity, or privileged-collector evidence
- Result: pass
- Reviewer/attestation: both recorded local gates passed; the initial record became visible after the first wrapper had returned model control, so a deterministic confirmation was retained rather than discarding either valid record; LF and CRLF canonical bytes were identical; external verifier copy SHA-256 `7D56967E130ED5EFF5372F4B7AE908A126429FDEE170B4E74DA4C80DCCAEB735` matched the in-tree built verifier before execution

## VER-BC03-HOSTED — BC-03 — 2026-07-19T03:16:02Z

- Source revision and dirty state: `d31e4a6e8126697357e7f0870f434ee24881e664`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29671499350, attempt 1
- Start/end/duration: `2026-07-19T03:16:02Z` / `2026-07-19T03:18:53Z` / 171 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88151233315 (Windows), 88151233342 (macOS Intel), 88151233351 (Linux), and 88151233352 (macOS arm64)
- Fixtures/manifests/bundle IDs: hosted schema gate exercised canonical JSON, the valid experiment manifest, and all three required-declaration rejection fixtures on every matrix job
- Counter availability and privileges: no physical counter evidence; workflow classification denies physical, energy, and long-duration claims
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and every matrix-job conclusion inspected through the GitHub API
