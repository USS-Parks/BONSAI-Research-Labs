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
## VER-BC07-ENV-FAIL — BC-07 — 2026-07-19T08:14:19Z

- Source revision and dirty state: `5542580c2f9870fa5f6d539a402b6577f898ca0e`; dirty with BC-07 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local sandbox; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; source locks unchanged
- Command: redacted machine record `BC-07-1784448859471416000`; full gate script invoked through system PowerShell without an execution-policy override
- Start/end/duration: `2026-07-19T08:14:19.471Z` / `2026-07-19T08:14:19.824Z` / 0.3528210 s
- Exit code: 1
- Stdout/stderr artifact hashes: empty stdout `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`; redacted stderr `9d2e0504f9d61e37fdda21eeb6ecb435a7feb8655f30ffb5ed7e82d7a7fced4c`
- Fixtures/manifests/bundle IDs: none executed; PowerShell blocked the local gate script before its first command
- Counter availability and privileges: not applicable; no elevation or policy persistence was requested
- Result: environment failure; contract gate not evaluated
- Reviewer/attestation: retained exact execution-policy denial; resolved in the following record with a process-local bypass and unchanged gate script

## VER-BC07-GATE — BC-07 — 2026-07-19T08:14:32Z

- Source revision and dirty state: `5542580c2f9870fa5f6d539a402b6577f898ca0e`; dirty with BC-07 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `BA7C806E67A42D80EC8D0D0D9781F937BEDA17B2B2B291C333BFDE5FFE9ABA04`
- Command: redacted machine record `BC-07-1784448872886496000`; full universal/schema/governance gate with uv cache redirected to ignored repository `target` storage and process-local PowerShell `-ExecutionPolicy Bypass`
- Start/end/duration: `2026-07-19T08:14:32.886Z` / `2026-07-19T08:14:49.466Z` / 16.5809302 s
- Exit code: 0
- Stdout/stderr artifact hashes: redacted stdout `ea1a9a532166f8250d24638690a3661be9b26cc7545cfd67386de2db07c9117b`; redacted stderr `7449e8d77f278cdd99ea805f2a921dccb5ab89a7feeaffdacb1f060f366c0cb8`
- Fixtures/manifests/bundle IDs: seven artifact types; complete consumer/cost/utility/revision trace; eight wrong-predecessor cases; missing birth/revision provenance; forbidden provenance cycles of lengths two through six; replaced/retired/removed terminal-resurrection cases; terminology registry negative fixture
- Counter availability and privileges: measured/estimated/unavailable states are contract fixtures only; no live counter, privileged collector, runtime lineage registry, learning internals, or physical evidence
- Result: pass
- Reviewer/attestation: strict artifact identity/revision/lifecycle/provenance validation, exact BC-07 property gate, registry coverage, and full automated local gate

## VER-BC08-GATE — BC-08 — 2026-07-19T08:32:33Z

- Source revision and dirty state: `9d0bd38b9a4b1aa1bce1823fd0a2f42a0dd755c4`; dirty with BC-08 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `BA7C806E67A42D80EC8D0D0D9781F937BEDA17B2B2B291C333BFDE5FFE9ABA04`
- Command: redacted machine record `BC-08-1784449953957691900`; full universal/schema/governance gate with repository-local uv cache and process-local PowerShell execution-policy bypass
- Start/end/duration: `2026-07-19T08:32:33.957Z` / `2026-07-19T08:32:43.994Z` / 10.0369853 s
- Exit code: 0
- Stdout/stderr artifact hashes: redacted stdout `c232171ce1fd1e7401f81b31bd88a61e1d1dd12131920bb1d83a7c058dd7e971`; redacted stderr `32ec1870131da130bcce39794f00a6795438fc632b95f672e93507c91ed9d29e`
- Fixtures/manifests/bundle IDs: four valid metric/uncertainty/claim fixtures; four explicit verdict states; `METRIC_SCALAR_UNIT_REQUIRED`, `METRIC_SCALAR_PROVENANCE_REQUIRED`, `CLAIM_CRITERION_REQUIRED`, `CLAIM_EVIDENCE_REQUIRED`, and `CLAIM_REASON_REQUIRED`
- Counter availability and privileges: all values are schema fixtures; no live metric computation, counter evidence, claim adjudication, privileged collector, or physical-host evidence
- Result: pass
- Reviewer/attestation: exact BC-08 scalar/provenance/criterion/evidence gate plus formula/version/unit/direction/population/window/estimator/missingness/precision/uncertainty/input/reason coverage and full automated local gate

## VER-BC08-HOSTED — BC-08 — 2026-07-19T08:36:33Z

- Source revision and dirty state: `1fd36aec8c4379ef594cb8cf18ea9be035af7870`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29680114105, attempt 1
- Start/end/duration: `2026-07-19T08:36:33Z` / `2026-07-19T08:39:03Z` / 150 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88174618112 (Linux), 88174618127 (macOS Intel), 88174618150 (Windows), and 88174618151 (macOS arm64)
- Fixtures/manifests/bundle IDs: hosted schema gate exercised all four valid metric/uncertainty/claim records, every rejection fixture, and all four explicit claim states on every matrix job
- Counter availability and privileges: no metric computation, claim adjudication, physical counter, privileged collector, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and every matrix-job conclusion inspected through the GitHub API

## VER-BC09-ENV-FAIL — BC-09 — 2026-07-19T08:56:21Z

- Source revision and dirty state: `1fd36aec8c4379ef594cb8cf18ea9be035af7870`; dirty with BC-09 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local sandbox; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; source locks included the BC-09 workspace update
- Command: machine record `BC-09-1784451381921850200`; full gate launched through the in-tree `cargo xtask verify` executable
- Start/end/duration: `2026-07-19T08:56:21.921Z` / `2026-07-19T08:56:41.595Z` / 19.6742214 s
- Exit code: 101
- Stdout/stderr artifact hashes: empty stdout `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`; stderr `3d41c471b558a533d892d35b698f389a026bded8199be1255933b82a461efb74`
- Fixtures/manifests/bundle IDs: format and strict workspace Clippy completed; workspace tests did not start because Cargo could not replace the still-running `target/debug/bonsai-xtask.exe`
- Counter availability and privileges: not applicable; no elevation or persistent policy change requested
- Result: environment/tooling failure; BC-09 corruption/recovery results did not fail
- Reviewer/attestation: exact Windows access-denied self-lock retained; resolved by building and checksum-matching an external verifier copy before rerunning the unchanged full gate

## VER-BC09-GATE — BC-09 — 2026-07-19T08:57:38Z

- Source revision and dirty state: `1fd36aec8c4379ef594cb8cf18ea9be035af7870`; dirty with BC-09 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `2B8C7C3C5687B4717246AB688EC6700C22FE42CA190053A4248ACFBCC7B302A9`
- Command: machine record `BC-09-1784451458102382400`; unchanged full universal/schema/governance gate through external verifier copy SHA-256 `BD1174BB50222583463195CEA005EA5B9A411AF56C5DE05CDF84DF2676CDB5A6`
- Start/end/duration: `2026-07-19T08:57:38.102Z` / `2026-07-19T08:57:53.183Z` / 15.0816955 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `ce492dcd4830a385675cc9a9f83211a03a3c2e6bdfb2ffecce8b4be4158f1f5f`; stderr `248e453649141d134e331cf7315a372ee49cf95cfa6b7723f00c3cbf8e3d0413`
- Fixtures/manifests/bundle IDs: `bonsai.event-segment/v1` matrix SHA-256 `6D4E776A41E0F13817F6EB91E5D7AAA218CD672BDF88FF41CFAC285331520AEB`; exact outcomes `SEGMENT_HEADER_TRUNCATED`, `SEGMENT_FRAME_TRUNCATED`, `SEGMENT_FRAME_CHECKSUM_MISMATCH`, `SEGMENT_CHECKSUM_MISMATCH`, `SEGMENT_FRAME_TOO_LARGE`, `BUNDLE_SEGMENT_SEQUENCE_DUPLICATE`, `BUNDLE_SEGMENT_SEQUENCE_NON_MONOTONIC`, `SEGMENT_RECOVERED`, and `SEGMENT_ALREADY_FINALIZED`
- Counter availability and privileges: no live event ingest, counter, agent, privileged collector, physical-host, energy, or long-duration evidence; format/recovery fixtures only
- Result: pass
- Reviewer/attestation: exact BC-09 truncation/corruption/size/sequence/recovery gate, strict workspace lint/tests, Python gates, schema compatibility, and all governance checks passed

## VER-BC09-HOSTED — BC-09 — 2026-07-19T09:05:03Z

- Source revision and dirty state: `26c093df265a3ae96089201140c74149ffd93caf`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29680948955, attempt 1
- Start/end/duration: `2026-07-19T09:05:03Z` / `2026-07-19T09:08:33Z` / 210 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88176902844 (Windows), 88176902855 (macOS Intel), 88176902856 (Linux), and 88176902879 (macOS arm64)
- Fixtures/manifests/bundle IDs: hosted Rust gate exercised every `bonsai.event-segment/v1` validation and recovery outcome on each OS family
- Counter availability and privileges: no live event ingest, physical counter, privileged collector, agent, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and every matrix-job conclusion inspected through the GitHub API

## VER-BC10-GATE — BC-10 — 2026-07-19T14:08:16Z

- Source revision and dirty state: `26c093df265a3ae96089201140c74149ffd93caf`; dirty with BC-10 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `79715B2EC269F5C5CC3E4B0755D2BB7870DDC6C6A8EB15197BF5252EAC15CAE8`
- Command: machine record `BC-10-1784470096942208100`; full universal/schema/governance gate through external verifier copy SHA-256 `E8BDEA495B4FDADCB63E16637023CFAC698BFB9C55043755FEBD0ADD65839E8F`
- Start/end/duration: `2026-07-19T14:08:16.942Z` / `2026-07-19T14:08:46.745Z` / 29.8036662 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `df787b8c90b703ae89a11c763340abe47227b501eef6c7a5fac9d9230703997a`; stderr `b9760d1c9f45a39f3acb9deba9891470198394138a23e009222d51e0e30387e5`
- Fixtures/manifests/bundle IDs: `bonsai.bundle-index/v1` matrix SHA-256 `ED9EE03878CE64037321F24523465F121CC5AA37DD7AB37AE7250697CF2000BE`; exact outcomes `BUNDLE_INDEX_REBUILT`, `BLOB_HASH_MISMATCH`, `BLOB_HASH_COLLISION`, `BLOB_ID_INVALID`, `BUNDLE_INDEX_PATH_INVALID`, and `BUNDLE_INDEX_READ_ONLY`; repeated file-only rebuild reproduced exact indexed rows
- Counter availability and privileges: portable storage and simulated fixture evidence only; no network database, agent access, live ingest, physical counter, privileged collector, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: exact BC-10 rebuild/hash/collision/traversal/read-only gate, strict workspace lint/tests, Python gates, schema compatibility, and all governance checks passed locally; hosted three-family evidence remains pending the focused published commit

## VER-BC10-HOSTED — BC-10 — 2026-07-19T14:14:10Z

- Source revision and dirty state: `5b64c01413abc4f7c6ae189e14e3e94d88380bb7`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29690432057, attempt 1
- Start/end/duration: `2026-07-19T14:14:10Z` / `2026-07-19T14:17:11Z` / 181 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88202053692 (macOS arm64), 88202053694 (macOS Intel), 88202053695 (Linux), and 88202053715 (Windows)
- Fixtures/manifests/bundle IDs: hosted Rust gate exercised `bonsai.bundle-index/v1` rebuild, hash/collision/traversal refusal, repeated row equivalence, and SQLite read-only enforcement on every OS family
- Counter availability and privileges: portable storage fixture evidence only; no network database, agent, physical counter, privileged collector, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and every matrix-job conclusion inspected through the GitHub API

## VER-BC11-GATE — BC-11 — 2026-07-19T14:34:09Z

- Source revision and dirty state: `5b64c01413abc4f7c6ae189e14e3e94d88380bb7`; dirty with BC-11 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `282632B786D3D1972A6B92379A47A658EBFBC97AFBB992C1B40C86B9F4C4FB0D`
- Command: machine record `BC-11-1784471649324168400`; full universal/schema/governance gate through external verifier copy SHA-256 `C021A501CFB1B84689457F8820866666A75B0C0726F5E0BBFA5FF5E8D942027E`
- Start/end/duration: `2026-07-19T14:34:09.324Z` / `2026-07-19T14:35:16.400Z` / 67.0774066 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `2612baef9494fab59c60ec5d92cc88d7514546bbdc963d2c6143c911afe73b09`; stderr `2ab885413b1a6ca9d0d624a9af41d2b9a25fab6fc2e7367985dea0fe034672c0`
- Fixtures/manifests/bundle IDs: `bonsai.derivation/v1` matrix SHA-256 `E0E03D4B39C7CA746C275F83A3B6919A174F5900910861CD5C2A2805FE371BE7`; exact outcomes `DERIVATION_TABLES_VALID`, `DERIVATION_SEMANTICALLY_IDENTICAL`, `DERIVATION_INPUT_MISMATCH`, and `DERIVATION_STALE`; all event/metric/lineage/decision tables round-tripped
- Counter availability and privileges: analytical fixture evidence only; no raw-evidence authority, metric computation, claim adjudication, live ingest, physical counter, privileged collector, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: exact BC-11 schema/provenance/semantic-regeneration/wrong-input/stale gate, strict workspace lint/tests, Python gates, schema compatibility, and all governance checks passed locally; hosted three-family evidence remains pending the focused published commit

## VER-BC11-HOSTED — BC-11 — 2026-07-19T14:38:16Z

- Source revision and dirty state: `7c483f3e0024da32163cc461c33d77162fc87156`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29691229681, attempt 1
- Start/end/duration: `2026-07-19T14:38:16Z` / `2026-07-19T14:42:34Z` / 258 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88204212057 (Linux), 88204212059 (macOS Intel), 88204212063 (Windows), and 88204212083 (macOS arm64)
- Fixtures/manifests/bundle IDs: hosted Rust gate exercised all four Arrow/Parquet table contracts, provenance validation, semantic regeneration equivalence, wrong-input detection, stale-producer detection, and no-replacement behavior on every OS family
- Counter availability and privileges: analytical fixture evidence only; no raw-evidence authority, metric computation, claim adjudication, live ingest, physical counter, privileged collector, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and every matrix-job conclusion inspected through the GitHub API

## VER-BC12-LINT-FAIL — BC-12 — 2026-07-19T15:06:07Z

- Source revision and dirty state: `7c483f3e0024da32163cc461c33d77162fc87156`; dirty with BC-12 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; source locks included the BC-12 workspace update
- Command: machine record `BC-12-1784473567251415900`; full gate launched through external verifier copy
- Start/end/duration: `2026-07-19T15:06:07.251Z` / `2026-07-19T15:06:10.975Z` / 3.723745 s
- Exit code: 101
- Stdout/stderr artifact hashes: stdout `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`; stderr `d997c4616ab4a34784f2a7d7e625150b0c4fa8819bd003616a69432eb744b7e8`
- Fixtures/manifests/bundle IDs: no corpus failure; strict Clippy rejected needless pass-by-value in the CLI argument handoff before tests
- Counter availability and privileges: no live bundle, physical counter, privileged collector, agent, energy, or long-duration evidence
- Result: implementation lint failure; corrected before acceptance
- Reviewer/attestation: retained as an honest failed attempt and not used as BC-12 acceptance evidence

## VER-BC12-GATE — BC-12 — 2026-07-19T15:07:15Z

- Source revision and dirty state: `7c483f3e0024da32163cc461c33d77162fc87156`; dirty with BC-12 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `2C9BA13AF614DB6DF5782166151E2CE0E85D97F72579998322023CB7EBA4B67C`
- Command: machine record `BC-12-1784473635637114200`; full universal/schema/governance gate through external verifier copy SHA-256 `86D52211F9B077D39A77A6967D970D14A196611ADF23D0CB59DE02A93FCE0659`
- Start/end/duration: `2026-07-19T15:07:15.637Z` / `2026-07-19T15:07:41.500Z` / 25.8643269 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `8491f96ae6af4d609c018149cf5f12119a471d98c158fc3ab545dacde4f3a3c3`; stderr `629ea33644568a9227c3d08256534bee21f777241140196153effc59a1f470b2`
- Fixtures/manifests/bundle IDs: `bonsai.bundle-validation-corpus/v1` matrix SHA-256 `6B55C615BE43BD4AA5B67B3AD83479C573CF923450ABDCC2831C797D51BAB2B6`; exact current, migratable, forward, corrupt, ambiguous-track, unavailable-counter, and tampered verdict/reason arrays; report schema SHA-256 `AA6C9F8A973A58EC1C4710860D3365FFC64EE01DB6FB751060EF56E09984ACC8`
- Counter availability and privileges: bundle fixture evidence only; unavailable-counter state remained explicit; no scientific claim adjudication, live agent, physical counter, privileged collector, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: exact BC-12 seven-case verdict corpus, deterministic non-mutating migration, forward read-only contract, one-report CLI semantics, strict workspace lint/tests, Python gates, schema compatibility, and all governance checks passed locally; hosted three-family evidence remains pending the focused published commit

## VER-BC12-HOSTED-FAIL — BC-12 — 2026-07-19T15:11:32Z

- Source revision and dirty state: `2cc885aad3a1ce153e7afc557224a42b34f79f6e`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29692334806, attempt 1
- Start/end/duration: `2026-07-19T15:11:32Z` / `2026-07-19T15:15:20Z` / 228 s
- Exit code: Linux and both macOS jobs concluded `success`; Windows concluded `failure` during Rust tests
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88207141611 (Windows, failure), 88207141625 (macOS arm64, success), 88207141631 (macOS Intel, success), and 88207141660 (Linux, success)
- Fixtures/manifests/bundle IDs: all three BC-12 corpus tests returned invalid only on the Windows checkout; Git had converted manifest-referenced JSON from LF to CRLF, changing exact stored-byte hashes without updating committed manifest identities
- Counter availability and privileges: bundle fixture evidence only; no live agent, physical counter, privileged collector, energy, or long-duration evidence
- Result: platform conformance failure; not acceptance evidence
- Reviewer/attestation: exact failing job log inspected; correction freezes repository JSON checkout bytes as LF and was reproduced through a separate `core.autocrlf=true` checkout before rerunning the complete gate

## VER-BC12-HOSTED — BC-12 — 2026-07-19T15:20:23Z

- Source revision and dirty state: `c8d03249920aa6ed98b353d9f046d84ddf8f3d66`; clean pushed revision equal to `origin/main`
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29692636701, attempt 1
- Start/end/duration: `2026-07-19T15:20:23Z` / `2026-07-19T15:24:38Z` / 255 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88207935347 (Windows), 88207935354 (macOS arm64), 88207935359 (Linux), and 88207935370 (macOS Intel)
- Fixtures/manifests/bundle IDs: all seven BC-12 bundle corpus cases retained exact expected verdicts with LF-stable stored-byte identities on every hosted OS
- Counter availability and privileges: bundle fixture evidence only; no live agent, physical counter, privileged collector, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and all four matrix-job conclusions inspected through the GitHub API; BC-12 is closed

## VER-BR01-GATE — BR-01 — 2026-07-19T16:07:51Z

- Source revision and dirty state: `c8d03249920aa6ed98b353d9f046d84ddf8f3d66`; dirty with BR-01 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `2C9BA13AF614DB6DF5782166151E2CE0E85D97F72579998322023CB7EBA4B67C`
- Command: machine record `BR-01-FINAL-1784477271044328200`; full universal/schema/governance gate through external verifier copy SHA-256 `9485AB2A886618734052ED79C6208BE1A43AC5B6CD3D51290CE86980BE4F4C0F`
- Start/end/duration: `2026-07-19T16:07:51.044Z` / `2026-07-19T16:08:46.200Z` / 55.1573395 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `6442ad3f111469e453463b12998fcbab7eae0e4780bd885788096328e98302d7`; stderr `fb3f2ad36f6dd7e817e5a713bdee56cae8c2c931eea5fd492469d27e151a327e`
- Fixtures/manifests/bundle IDs: `bonsai.adapter-protocol-outcomes/v1` SHA-256 `E0CD82C0C1DD6C6927E62EE56555FA330B2C4D52A0E8E0321E95299B4AEF85FB`; adapter schema SHA-256 `0B01498A4D6DE375D03CE00A51FEA909F4D111F3EDB1D5BE5E31049385042D0D`; exact valid, explicit-flag, out-of-order, version-mismatch, capability-change, and post-stop classes
- Counter availability and privileges: schema/state-machine fixture evidence only; no live child process, inherited-handle transport, physical-host process spot check, privileged input, observer isolation, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: complete declared operation path, deterministic seeds/deadlines, frozen capability negotiation, fail-closed state/sequence behavior, strict workspace gates, Python gates, schema compatibility, and all governance checks passed locally; hosted Rust-Python process evidence belongs to BR-02

## VER-BR01-HOSTED — BR-01 — 2026-07-19T16:12:47Z

- Source revision and dirty state: `f8bc73b158a3a407cd7e252c76cfbeddfcce2654`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29694408721, attempt 1
- Start/end/duration: `2026-07-19T16:12:47Z` / `2026-07-19T16:16:43Z` / 236 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88212548048 (Windows), 88212548057 (Linux), 88212548062 (macOS arm64), and 88212548072 (macOS Intel)
- Fixtures/manifests/bundle IDs: BR-01 state-machine and explicit capability-presence conformance ran unchanged on every hosted OS
- Counter availability and privileges: schema/state-machine fixture evidence only; no child-process transport, physical-host process spot check, privileged input, observer isolation, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and all four matrix-job conclusions inspected through the GitHub API

## VER-BR04-GATE — BR-04 — 2026-07-19T17:01:28Z

- Source revision and dirty state: `e406d26157fc36cff4b9c02e55eeb425ce93bfbc`; dirty with BR-04 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `FF83EB84B00CA8FC4DE402433CA839A8DCDBBE6D817FB3A82832D4B3E1D8587E`
- Command: machine record `BR-04-1784480488552125000`; full universal/schema/governance gate through external verifier copy SHA-256 `F0889F66D31EA0987A099F036DDB235CC0A0F31474ABF6322B227EE8623A1ED6`
- Start/end/duration: `2026-07-19T17:01:28.552Z` / `2026-07-19T17:02:01.537Z` / 32.9853869 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `534afbcea4b6c1fc2c2d64b06d47de6dddb389093fc19b30f0f8f289aedde785`; stderr `81d5e8987a260c7755cbaa481df2f40347b3d2dad7f95d3226886b7faa63bf55`
- Fixtures/manifests/bundle IDs: `bonsai.event-ordering-outcomes/v1` SHA-256 `065C59C29629C6CEB9F95378FEA5259B17DB67ECA7ADADB8E756B0FF7A2D3DA4`; exact source/causal/late/duplicate/missing/concurrent/regression/conflict/gap/cycle classes and collection-order invariance
- Counter availability and privileges: deterministic graph fixtures only; wall time explicitly non-authoritative; no run recovery, physical counter, process isolation, privileged input, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: identity-sorted partial-order semantics, bounded graph construction, randomized collection equality, no fabricated wall-time/ambiguous edges, strict workspace/Python/schema/governance gates passed locally

## VER-BR02-GATE — BR-02 — 2026-07-19T16:31:42Z

- Source revision and dirty state: `f8bc73b158a3a407cd7e252c76cfbeddfcce2654`; dirty with BR-02 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `86692A6AB0CA6837B8187AA5F07B73B3B252CA6913991763CBFA1213879B15EC`
- Command: machine record `BR-02-FINAL-1784478702679705900`; full universal/schema/governance gate through external verifier copy SHA-256 `D955F1864AB9924DBADC0AF424C20F9F54E3271A9C0C10362C1EFFD8DFA67950`
- Start/end/duration: `2026-07-19T16:31:42.680Z` / `2026-07-19T16:32:27.340Z` / 44.6618344 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `e12cad842afd2a79b29ec7c7f5ea25953e713fb1a275b2051561de47546b222a`; stderr `e32b61c6e14dcaf42ec4fa6e0ecf931a3bc59255fa812c4184df323c0b9a876d`
- Fixtures/manifests/bundle IDs: `bonsai.process-transport-outcomes/v1` SHA-256 `ABF7571E9E955750A4530BCB72B1A9D56BF0BA2E78FA818D8A342FFB6EBEA75F`; exact clean echo/shutdown, partial, oversized, stalled, and flood outcomes
- Counter availability and privileges: live local child-process pipe behavior with physical/virtual status unknown; no descendant resource enforcement, filesystem isolation, physical-host attestation, privileged input, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: strict Rust and Python gates, real cross-language inherited pipes, fixed framing/allocation/queue/log bounds, timeout/flood containment records, clean shutdown, schema compatibility, and governance checks passed locally; hosted Windows/macOS/Linux conformance remains pending the focused published commit

## VER-BR02-HOSTED — BR-02 — 2026-07-19T16:36:23Z

- Source revision and dirty state: `380b9764ea4c9cf6c80a9fadd2cdea75eb98aa9e`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29695193065, attempt 1
- Start/end/duration: `2026-07-19T16:36:23Z` / `2026-07-19T16:40:21Z` / 238 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88214621181 (Linux), 88214621187 (macOS arm64), 88214621191 (macOS Intel), and 88214621205 (Windows)
- Fixtures/manifests/bundle IDs: real Rust-to-Python/Python-to-Rust pipe exchange plus clean, partial, oversized, stalled, and flood process outcomes ran unchanged on every hosted OS
- Counter availability and privileges: hosted child-process behavior only; no physical-host process attestation, descendant resource enforcement, filesystem isolation, privileged input, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and all four matrix-job conclusions inspected through the GitHub API; BR-02 cross-OS gate is closed

## VER-BR03-GATE — BR-03 — 2026-07-19T16:47:21Z

- Source revision and dirty state: `380b9764ea4c9cf6c80a9fadd2cdea75eb98aa9e`; dirty with BR-03 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `FF83EB84B00CA8FC4DE402433CA839A8DCDBBE6D817FB3A82832D4B3E1D8587E`
- Command: machine record `BR-03-1784479641202426700`; full universal/schema/governance gate through external verifier copy SHA-256 `73DD6ADBD09CCB3AB8E95E4F664EF349896C1CC2480DD4B386E0185FEF74B484`
- Start/end/duration: `2026-07-19T16:47:21.202Z` / `2026-07-19T16:47:55.825Z` / 34.6235 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `0dbeb1c40a69101ce6e86c9bc5f3d81348ee0ef5814ce719b5c88afe9b37d39c`; stderr `36aa3fe3031538547a8a4ccc8444f47c173d06ba1d89e15bbfe16918a5e8f040`
- Fixtures/manifests/bundle IDs: `bonsai.event-ingest-outcomes/v1` SHA-256 `7B290AF924E1334463C9B95F3D643439C8DE9B2C5E8913701F7A261A00BEFDEB`; exact valid, run/source/type/schema/hash/rate/lifecycle outcomes plus deterministic 2,048-input fuzz corpus
- Counter availability and privileges: live local immutable segment writes and observer-arrival rate fixtures with physical/virtual status unknown; no ordering claim, process isolation, physical counter, privileged input, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: validation-before-append, original-byte preservation, stable contract errors, bounded source/schema/size/rate/lifecycle enforcement, bounded rejection evidence, panic-free fuzz corpus, strict workspace/Python/schema/governance gates passed locally

## VER-BR03-HOSTED — BR-03 — 2026-07-19T16:51:15Z

- Source revision and dirty state: `e406d26157fc36cff4b9c02e55eeb425ce93bfbc`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29695677005, attempt 1
- Start/end/duration: `2026-07-19T16:51:15Z` / `2026-07-19T16:55:02Z` / 227 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88215904321 (macOS arm64), 88215904328 (Linux), 88215904330 (macOS Intel), and 88215904367 (Windows)
- Fixtures/manifests/bundle IDs: exact ingest acceptance/rejection matrix, bounded observer ledger, and deterministic fuzz corpus ran unchanged on every hosted OS
- Counter availability and privileges: hosted immutable-segment and validation fixtures only; no physical-host attestation, ordering claim, process isolation, privileged input, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and all four matrix-job conclusions inspected through the GitHub API

## VER-BR04-HOSTED — BR-04 — 2026-07-19T17:06:15Z

- Source revision and dirty state: `9b02dbe46ea2b82d11f613de38be074c08aff69b`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29696164878, attempt 1
- Start/end/duration: `2026-07-19T17:06:15Z` / `2026-07-19T17:10:36Z` / 261 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88217197816 (macOS Intel), 88217197818 (macOS arm64), 88217197824 (Linux), and 88217197832 (Windows)
- Fixtures/manifests/bundle IDs: exact partial-order class/edge/concurrency matrix and collection-order invariance ran unchanged on every hosted OS
- Counter availability and privileges: hosted deterministic graph fixtures only; wall time explicitly non-authoritative; no run recovery, physical counter, process isolation, privileged input, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and all four matrix-job conclusions inspected through the GitHub API; BR-04 cross-OS gate is closed

## VER-BR05-PRETIGHTENING — BR-05 — 2026-07-19T17:21:19Z

- Source revision and dirty state: `9b02dbe46ea2b82d11f613de38be074c08aff69b`; dirty with BR-05 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `FA903972D3ECBF809923DE9B6FBD3CB1C3C456307945BC20B10A750321EC74B6`
- Command: machine record `BR-05-1784481679257337900`; full universal/schema/governance gate through external verifier copy SHA-256 `301B89F7ACED8B7061844EDF7BC6B98A4A3A562D182BB817AB3498A20E1910C8`
- Start/end/duration: `2026-07-19T17:21:19.257Z` / `2026-07-19T17:21:41.021Z` / 21.7647429 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `0c9cba70805a60d890fc34e2b7c92cb56b8bc6865f263be91efae71719938239`; stderr `faac86601da06c29960ad2cfb12f6525274e757008ef18c8bc79c97f3b71d985`
- Fixtures/manifests/bundle IDs: `bonsai.run-lifecycle-outcomes/v1` SHA-256 `89AFD8FE20B527872EA07903133256B6FBD2176DD9B70F1861D77A69BE24098F`; every lifecycle and transition-commit crash boundary, exact segment count, terminal state, and no-agent-resume outcome
- Counter availability and privileges: local filesystem durability/recovery fixtures only; no adapter restart, observer replay, physical-host attestation, resource enforcement, privileged input, energy, or long-duration evidence
- Result: pass; superseded as final acceptance evidence
- Reviewer/attestation: all checks passed, but post-gate review distinguished `BufWriter` append/drop behavior from an explicit durable crash boundary; the record remains historical evidence and the final record below verifies the added flush-and-sync boundary

## VER-BR05-GATE — BR-05 — 2026-07-19T17:26:07Z

- Source revision and dirty state: `9b02dbe46ea2b82d11f613de38be074c08aff69b`; dirty with final BR-05 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `FA903972D3ECBF809923DE9B6FBD3CB1C3C456307945BC20B10A750321EC74B6`
- Command: machine record `BR-05-FINAL-1784481967656742400`; full universal/schema/governance gate through external verifier copy SHA-256 `9C6D8003759EB555122B87211203A919AC930A8133908AD4DC825BC9772C958F`
- Start/end/duration: `2026-07-19T17:26:07.656Z` / `2026-07-19T17:26:34.670Z` / 27.0141562 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `52fe49bf86a736d74e79787d28d86eb3a40d4f1965f86314274506d759e3ffc3`; stderr `7657b7471678f5c4d23a70caab908ecb86b92b892dc4ebb5409746542cd3f596`
- Fixtures/manifests/bundle IDs: `bonsai.run-lifecycle-outcomes/v1` SHA-256 `89AFD8FE20B527872EA07903133256B6FBD2176DD9B70F1861D77A69BE24098F`; every lifecycle and durable transition-commit crash boundary, exact segment count, terminal state, and no-agent-resume outcome
- Counter availability and privileges: local synchronized-filesystem recovery fixtures only; no adapter restart, observer replay, physical-host attestation, resource enforcement, privileged input, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: legal lifecycle graph, synchronized journal and complete pending frame, copy-and-publish segment recovery, immutable transition settlement, exact once-only segment counts, strict workspace/Python/schema/governance gates passed locally; hosted Windows/macOS/Linux closure remains attached to the focused commit

## VER-BR05-HOSTED — BR-05 — 2026-07-19T17:30:01Z

- Source revision and dirty state: `fa7096a238ed96889a0156aea45e3a69f7dab807`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29696943312, attempt 1
- Start/end/duration: `2026-07-19T17:30:01Z` / `2026-07-19T17:34:44Z` / 283 s
- Exit code: all four jobs concluded `success`
- Stdout/stderr artifact hashes: retained by GitHub Actions; job IDs 88219222167 (macOS arm64), 88219222191 (Windows), 88219222192 (Linux), and 88219222200 (macOS Intel)
- Fixtures/manifests/bundle IDs: exact lifecycle and durable transition-commit crash boundaries, synchronized staged-frame recovery, terminal evidence, and once-only segment counts ran unchanged on every hosted OS
- Counter availability and privileges: hosted synchronized-filesystem recovery fixtures only; no adapter restart, observer replay, physical-host attestation, resource enforcement, privileged input, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: run head SHA, run conclusion, and all four matrix-job conclusions inspected through the GitHub API; BR-05 cross-OS gate is closed

## VER-BR06-TYPE-FAILURE — BR-06 — 2026-07-19T17:41:07Z

- Source revision and dirty state: `fa7096a238ed96889a0156aea45e3a69f7dab807`; dirty with BR-06 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; source locks at the working revision
- Command: machine record `BR-06-1784482867979810300`; full universal/schema/governance gate through an external verifier copy
- Start/end/duration: `2026-07-19T17:41:07.979Z` / `2026-07-19T17:41:27.753Z` / 19.7742117 s
- Exit code: 1
- Stdout/stderr artifact hashes: stdout `2e2d79ff49611eaf9648fee987a8ec3ea2d6f4a6607383eff8cdd8bbdd863cb8`; stderr `bdb6f7b2fbb39f4fbc3db3d4701c0852992f69c20ae1aa78732c86f8a0b23811`
- Fixtures/manifests/bundle IDs: product isolation tests and all preceding repository gates passed; Pyright rejected the inspection adapter result as partially unknown
- Counter availability and privileges: local child-process/interface fixture only; no physical-host attestation or OS sandbox claim
- Result: fail; exact strict-type failure retained
- Reviewer/attestation: the heterogeneous result dictionary lacked an explicit structural type; fixed with a precise `TypedDict` and rerun below without relaxing Pyright

## VER-BR06-PRE-HASH-TIGHTENING — BR-06 — 2026-07-19T17:42:19Z

- Source revision and dirty state: `fa7096a238ed96889a0156aea45e3a69f7dab807`; dirty with final BR-06 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `AFE32A618E77FEE0405535C2087B46FF05E31AA00D8BAC7C6369BE4C6CC9BF00`
- Command: machine record `BR-06-FINAL-1784482939010080200`; full universal/schema/governance gate through external verifier copy SHA-256 `08FED2EB1B517AB584F2D2A427A91D25C1BCD086314EFC0480B4A0AB151DA2A6`
- Start/end/duration: `2026-07-19T17:42:19.010Z` / `2026-07-19T17:42:38.848Z` / 19.8388921 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `4d5e8109ba6a0c14043d11c9f93834dbf29e76a3eb58acfca55f9b68e4285620`; stderr `3cd4bedd5dce611d741a340354ba4d78cd4c3a28055628ea91b438c3ba5174bd`
- Fixtures/manifests/bundle IDs: `bonsai.agent-isolation-outcomes/v1` SHA-256 `56600753C15618042E4192A0A8DBDD6253AD238948EB330228967E88AE4780D1`; exact cleared-environment, granted-input/work/handle, observer-path rejection, denial, and derived-track outcomes
- Counter availability and privileges: real local environment-cleared Python child with granted standard streams and filesystem roots; physical/virtual status unknown; no native-code OS sandbox, physical-host attestation, privileged input, energy, or long-duration evidence
- Result: pass; superseded as final acceptance evidence
- Reviewer/attestation: all checks passed, but post-gate review strengthened the caller-authorized copy into an exact manifest-SHA-256-bound grant; this record remains historical and the final record below verifies the tighter contract

## VER-BR06-PRE-READONLY-TIGHTENING — BR-06 — 2026-07-19T17:46:40Z

- Source revision and dirty state: `fa7096a238ed96889a0156aea45e3a69f7dab807`; dirty with final BR-06 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `AFE32A618E77FEE0405535C2087B46FF05E31AA00D8BAC7C6369BE4C6CC9BF00`
- Command: machine record `BR-06-FINAL2-1784483200000031500`; full universal/schema/governance gate through external verifier copy SHA-256 `F0D5A190C1F99681023C91362CB37FFAB9D39F51E14C65989588A5DF8C8AEFFB`
- Start/end/duration: `2026-07-19T17:46:40.000Z` / `2026-07-19T17:47:14.666Z` / 34.6671599 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `fb0e4b66276e8ac878e341413abef3446ff9e2279906ca7e119cfeb4cd062511`; stderr `c94e18b124b1b0768e3c979bf85e9c8e64b87c693c8a8b16984cebe36ad1c224`
- Fixtures/manifests/bundle IDs: `bonsai.agent-isolation-outcomes/v1` SHA-256 `2111D422DB6A95455C0E2C9BE62F5EF94BA63BF51F92D89EC1FBFD0B238C9DF0`; exact cleared-environment, manifest-hash-bound input, rejected-copy cleanup, agent-work/handle, observer-path rejection, denial, and derived-track outcomes
- Counter availability and privileges: real local environment-cleared Python child with granted standard streams and filesystem roots; physical/virtual status unknown; no native-code OS sandbox, physical-host attestation, privileged input, energy, or long-duration evidence
- Result: pass; superseded as final acceptance evidence
- Reviewer/attestation: all checks passed, but final review aligned the filesystem mode with the declared input/work split by marking accepted input copies read-only; this record remains historical and the final record below verifies that mode

## VER-BR06-GATE — BR-06 — 2026-07-19T17:50:46Z

- Source revision and dirty state: `fa7096a238ed96889a0156aea45e3a69f7dab807`; dirty with final BR-06 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `AFE32A618E77FEE0405535C2087B46FF05E31AA00D8BAC7C6369BE4C6CC9BF00`
- Command: machine record `BR-06-FINAL3-1784483446064476700`; full universal/schema/governance gate through external verifier copy SHA-256 `912E199D28394696E858A152DBCB691EC42995F087B5B8EE4D1BF24060C9DEFA`
- Start/end/duration: `2026-07-19T17:50:46.064Z` / `2026-07-19T17:51:06.278Z` / 20.2144321 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `f895275a4154a5be8042014f849e8499d8f00d68c0b30b89c79de0c1cad8626f`; stderr `a468062dcee70af1f7cc40ced027f1a10abcc28fbd7baad3135a7318a3ab1041`
- Fixtures/manifests/bundle IDs: `bonsai.agent-isolation-outcomes/v1` SHA-256 `D6C45C991A8F88C0D44AB2CE81C010CA6B39DF47E94ABE256EC8F646C382CC5C`; exact cleared-environment, manifest-hash-bound read-only input, rejected-copy cleanup, agent-work/handle, observer-path rejection, denial, and derived-track outcomes
- Counter availability and privileges: real local environment-cleared Python child with granted standard streams and filesystem roots; physical/virtual status unknown; no native-code OS sandbox, physical-host attestation, privileged input, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: manifest-SHA-256-bound read-only input copying, agent work root, argument/environment/handle/protocol audit, real inspection child, observer canary non-discovery through granted interfaces, mismatch cleanup, explicit denial evidence, BC-05 non-Track-A derivation, strict workspace/Python/schema/governance gates passed locally; hosted Windows/macOS/Linux closure remains attached to the focused commit

## VER-BR06-HOSTED-FAILURE — BR-06 — 2026-07-19T17:56:30Z

- Source revision and dirty state: `e878be07653c524a08997c78a583ff2457fbcfd4`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29697809624, attempt 1
- Start/end/duration: `2026-07-19T17:56:30Z` / `2026-07-19T18:01:02Z` / 272 s
- Exit code: Windows job 88221478201 and Linux job 88221478217 concluded `success`; macOS arm64 job 88221478199 and macOS Intel job 88221478194 concluded `failure`
- Stdout/stderr artifact hashes: retained by GitHub Actions; both macOS jobs passed the first four BR-06 tests, then the real inspection adapter exceeded the test-only 5-second receive allowance and returned `TRANSPORT_READ_TIMEOUT`
- Fixtures/manifests/bundle IDs: all non-inspection isolation outcomes passed on every hosted OS; the inspection outcome passed Windows/Linux and timed out before returning a result on both macOS architectures
- Counter availability and privileges: hosted child-process/interface fixture only; no physical-host or OS sandbox claim
- Result: fail; exact cross-platform timeout retained
- Reviewer/attestation: failure is isolated to a too-short test process-start/response allowance, not an observer exposure or classification mismatch; correction retains a finite 20-second bound and changes no product semantics

## VER-BR06-CORRECTION-GATE — BR-06 — 2026-07-19T18:01:31Z

- Source revision and dirty state: `e878be07653c524a08997c78a583ff2457fbcfd4`; dirty only with the BR-06 portability correction and its evidence
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `AFE32A618E77FEE0405535C2087B46FF05E31AA00D8BAC7C6369BE4C6CC9BF00`
- Command: machine record `BR-06-CORRECTION-1784484091887149900`; full universal/schema/governance gate through external verifier copy SHA-256 `D77C29AB079F91BF171AB5C7FC806DAAEB7ABD09FB603865E34DA54AF0D20169`
- Start/end/duration: `2026-07-19T18:01:31.887Z` / `2026-07-19T18:02:07.574Z` / 35.6886645 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `e20e66bd15f95fe030f107d22487f1d412d1c2cb0b79a91c1186bc1650c93a0a`; stderr `a40921d505217f44905546ecb50f23f753b50743d9b9b1981fee74ec2155f75a`
- Fixtures/manifests/bundle IDs: unchanged `bonsai.agent-isolation-outcomes/v1`; real inspection child passed 10/10 focused repetitions before the full gate
- Counter availability and privileges: real local environment-cleared Python child with a finite 20-second process allowance; no native-code OS sandbox, physical-host attestation, privileged input, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: correction changes only the test receive/shutdown allowance from 5 to 20 seconds; strict workspace/Python/schema/governance gates pass and hosted rerun remains required

## VER-BR06-CORRECTION-HOSTED-FAILURE — BR-06 — 2026-07-19T18:05:40Z

- Source revision and dirty state: `5785a09fe1db3e2f63b291934531e1915c81121b`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29698103848, attempt 1
- Start/end/duration: `2026-07-19T18:05:40Z` / `2026-07-19T18:09:41Z` / 241 s
- Exit code: Windows job 88222227394 and Linux job 88222227395 concluded `success`; macOS arm64 job 88222227377 and macOS Intel job 88222227387 concluded `failure`
- Stdout/stderr artifact hashes: retained by GitHub Actions; macOS arm64 exited without a frame after 6.61 seconds under the 20-second allowance, proving this was not process-start timeout
- Fixtures/manifests/bundle IDs: all non-inspection isolation outcomes passed on every hosted OS; the inspection process failed only while traversing its broader ambient environment-derived candidate roots on macOS
- Counter availability and privileges: hosted child-process/interface fixture only; no physical-host or OS sandbox claim
- Result: fail; exact incomplete first correction retained
- Reviewer/attestation: the inspection fixture exceeded the granted-interface model by treating every ambient OS environment value as a filesystem grant; refinement restricts discovery to configured BONSAI roots and explicit arguments and adds child-stderr diagnostics

## VER-BR06-CORRECTION2-GATE — BR-06 — 2026-07-19T18:10:38Z

- Source revision and dirty state: `5785a09fe1db3e2f63b291934531e1915c81121b`; dirty only with the refined BR-06 portability correction and its evidence
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python 3.14.4; uv 0.11.29; `Cargo.lock` SHA-256 `AFE32A618E77FEE0405535C2087B46FF05E31AA00D8BAC7C6369BE4C6CC9BF00`
- Command: machine record `BR-06-CORRECTION2-1784484638059258600`; full universal/schema/governance gate through external verifier copy SHA-256 `46B4426A145BB2E84C2A9694A4A0BB493CC105460A3D8031E6751F880A6CADA4`
- Start/end/duration: `2026-07-19T18:10:38.059Z` / `2026-07-19T18:11:03.246Z` / 25.1876089 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `b4b15c60f795f9eacef5ec9aa31774341639dec0381b5da4d2d5e3d5a940698f`; stderr `7225ea5c2a4114ebfef5d397c042d30d8064247661154e6f490e4ba7d7e6a87c`
- Fixtures/manifests/bundle IDs: unchanged `bonsai.agent-isolation-outcomes/v1`; real inspection child passed 10/10 focused repetitions before the full gate
- Counter availability and privileges: real local environment-cleared Python child enumerating only configured BONSAI roots and explicit grants; no native-code OS sandbox, physical-host attestation, privileged input, energy, or long-duration evidence
- Result: pass
- Reviewer/attestation: deterministic non-symlink discovery, strict Ruff/Pyright, diagnostic child-exit evidence, strict workspace/schema/governance gates pass; hosted rerun remains required

## VER-BR06-CORRECTION2-HOSTED — BR-06 — 2026-07-19T18:15:08Z

- Source revision and dirty state: `65c1d3e778790a4c679c904242d397af675dbcbb`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29698386866, attempt 1
- Start/end/duration: `2026-07-19T18:15:08Z` / `2026-07-19T18:19:22Z` / 254 s
- Exit code: all four jobs concluded `success`; job IDs 88222990574 (Linux), 88222990580 (Windows), 88222990585 (macOS arm64), and 88222990601 (macOS Intel)
- Stdout/stderr artifact hashes: retained by GitHub Actions
- Fixtures/manifests/bundle IDs: unchanged BR-06 agent-isolation outcome matrix, including the real inspection child constrained to granted roots
- Counter availability and privileges: hosted child-process/interface fixture only; no physical-host or OS sandbox claim
- Result: pass
- Reviewer/attestation: exact head SHA, workflow conclusion, and all four matrix-job conclusions inspected through the GitHub API; BR-06 cross-OS gate is closed

## VER-BM01-INVOKE-FAILURE — BM-01 — 2026-07-19T18:32:13Z

- Source revision and dirty state: `65c1d3e778790a4c679c904242d397af675dbcbb`; dirty with BM-01 changes
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; Python environment locked by `uv.lock`; source locks at the working revision
- Command: machine record `BM-01-1784485933992330400`; initial nested PowerShell full-gate invocation
- Start/end/duration: machine-record Unix UTC nanoseconds `1784485933992330400` / `1784485953676180200` / 19.6843757 s
- Exit code: 1
- Stdout/stderr artifact hashes: stdout `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855`; stderr `7EBB726B623BC68350167F994133B78A21BF36A42D231D7B3D97759B9F049ECF`
- Fixtures/manifests/bundle IDs: no product fixture failure; outer PowerShell stripped nested command variables, and the in-use verifier binary could not be replaced by Cargo
- Counter availability and privileges: not applicable
- Result: fail; invocation defect retained
- Reviewer/attestation: rerun below uses an external verifier copy, quoted nested command, and explicit locked-environment executables

## VER-BM01-GATE — BM-01 — 2026-07-19T18:33:49Z

- Source revision and dirty state: `65c1d3e778790a4c679c904242d397af675dbcbb`; dirty only with BM-01 implementation, governance, and evidence
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; locked repository Python environment; `Cargo.lock` SHA-256 `9D7BEB095C99C48C375F96F2E5016304B3837523C494EFEE07BF24B16E2E28AF`
- Command: machine record `BM-01-FINAL-1784486029814026000`; full universal/schema/governance gate through an external verifier copy
- Start/end/duration: machine-record Unix UTC nanoseconds `1784486029814026000` / `1784486057501489800` / 27.6882134 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `28121038F27417CA14642343E27E5289D70404FB0E4AC21F23B5620EFA932887`; stderr `49405EF5633A683E91813A5D1D61C39361AA95648F2EE9461A8AD7BFF2CF02F3`
- Fixtures/manifests/bundle IDs: four BM-01 focused tests plus complete existing repository corpus
- Counter availability and privileges: contract fixtures only; no live platform collector, physical-host, privileged counter, energy, or enforcement claim
- Result: pass
- Reviewer/attestation: measured/estimated/unavailable/error invariants, zero-versus-missingness, exact advertised-counter coverage, strict workspace/Python/schema/governance gates passed locally; hosted closure remains attached to the focused commit

## VER-BM01-HOSTED — BM-01 — 2026-07-19T18:41:16Z

- Source revision and dirty state: `a6387e31f80236e210d355d312bd02f84fa17103`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29699140238, attempt 1
- Start/end/duration: `2026-07-19T18:41:16Z` / `2026-07-19T18:48:10Z` / 414 s
- Exit code: all four jobs concluded `success`; job IDs 88225065797 (Linux), 88225065801 (macOS arm64), 88225065811 (macOS Intel), and 88225065815 (Windows)
- Stdout/stderr artifact hashes: retained by GitHub Actions
- Fixtures/manifests/bundle IDs: four BM-01 sample/coverage tests plus the complete prior repository corpus
- Counter availability and privileges: contract fixtures only; no live platform collector, physical-host, privileged counter, energy, or enforcement claim
- Result: pass
- Reviewer/attestation: exact head SHA, workflow conclusion, and all four matrix-job conclusions inspected through the GitHub API; BM-01 cross-OS gate is closed

## VER-BM02-GATE — BM-02 — 2026-07-19T18:44:58Z

- Source revision and dirty state: `a6387e31f80236e210d355d312bd02f84fa17103`; dirty only with BM-02 implementation, governance, and evidence
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; locked repository Python environment; `Cargo.lock` unchanged from BM-01
- Command: machine record `BM-02-1784486698921379800`; full universal/schema/governance gate through an external verifier copy
- Start/end/duration: machine-record Unix UTC nanoseconds `1784486698921379800` / `1784486729353099300` / 30.4325617 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `A3698E36F2497D1B2B0C8B491B3833FE2DDB43DBDB385934D6CF17FB03258DA1`; stderr `3EB9F3C9CC7823821944B6D4FCC64B91206BF90BFA1DB3AE3FB086AC9BA2B6CA`
- Fixtures/manifests/bundle IDs: deterministic regression/suspend fixtures and live 256-probe system-clock calibration plus complete existing repository corpus
- Counter availability and privileges: process-local standard-library clocks only; wall comparison is optional and cross-process comparison remains unqualified; no physical-host precision or privileged counter claim
- Result: pass
- Reviewer/attestation: monotonic duration authority, effective resolution, bracketed call overhead, regression/suspend annotations, strict workspace/Python/schema/governance gates passed locally; hosted closure remains attached to the focused commit

## VER-BM02-HOSTED — BM-02 — 2026-07-19T18:51:43Z

- Source revision and dirty state: `4c9b88f937a019597dfe1a733f718f67dbb5bf4d`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29699440769, attempt 1
- Start/end/duration: `2026-07-19T18:51:43Z` / `2026-07-19T18:56:02Z` / 259 s
- Exit code: all four jobs concluded `success`; job IDs 88225862066 (macOS arm64), 88225862068 (Windows), 88225862101 (Linux), and 88225862104 (macOS Intel)
- Stdout/stderr artifact hashes: retained by GitHub Actions
- Fixtures/manifests/bundle IDs: deterministic clock regression/suspend fixtures and live system-clock calibration on every hosted runner
- Counter availability and privileges: process-local standard-library clocks only; no physical-host precision or cross-process equivalence claim
- Result: pass
- Reviewer/attestation: exact head SHA, workflow conclusion, and all four matrix-job conclusions inspected through the GitHub API; BM-02 cross-OS gate is closed

## VER-BM03-GATE — BM-03 — 2026-07-19T19:00:27Z

- Source revision and dirty state: `4c9b88f937a019597dfe1a733f718f67dbb5bf4d`; dirty only with BM-03 implementation, dependencies, governance, and evidence
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; locked repository Python environment; `sysinfo` 0.39.6; `Cargo.lock` SHA-256 `A7D1D8F57EB4553059F34EBAE1E487F0FB69DB1686FCB5F0082DD1F1366F52B8`
- Command: final machine record `BM-03-FINAL-1784487627468498600`; full universal/schema/governance gate through an external verifier copy; earlier full and focused passing records retained
- Start/end/duration: machine-record Unix UTC nanoseconds `1784487627468498600` / `1784487655594010000` / 28.1262745 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `1B72F9F80044ED457072B749355AAE37BF7E8558E02C1FA653C82424F137D6B6`; stderr `E6579DF6023A5F9ECED842622EFE45D3C9B3EDFB9E9B965B85949823743A81D4`
- Fixtures/manifests/bundle IDs: real parent/child process-tree fixture, exact live agent/observer storage totals, exact operation counters, overflow failure, and complete existing repository corpus
- Counter availability and privileges: live unprivileged local process and filesystem counters; RSS/virtual-memory/I/O semantics are platform-qualified; no platform hard-cap, physical-host, or energy claim
- Result: pass
- Reviewer/attestation: real descendant inclusion, CPU/memory/I/O aggregate shape, exact scoped storage, checked work counters, strict workspace/Python/schema/governance gates passed locally; hosted closure remains attached to the focused commit

## VER-BM03-HOSTED — BM-03 — 2026-07-19T19:04:03Z

- Source revision and dirty state: `b3b95618deca53f1938915aa879c10ced3f5ae15`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; `sysinfo` 0.39.6; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29699889177, attempt 1
- Start/end/duration: `2026-07-19T19:04:03Z` / `2026-07-19T19:08:56Z` / 293 s
- Exit code: all four jobs concluded `success`; job IDs 88227039609 (macOS arm64), 88227039612 (macOS Intel), 88227039615 (Windows), and 88227039636 (Linux)
- Stdout/stderr artifact hashes: retained by GitHub Actions
- Fixtures/manifests/bundle IDs: real parent/child process tree, scoped live storage, exact operation counters, and complete prior repository corpus
- Counter availability and privileges: live hosted unprivileged process/filesystem counters; no physical-host, hard-cap, or energy claim
- Result: pass
- Reviewer/attestation: exact head SHA, workflow conclusion, and all four matrix-job conclusions inspected through the GitHub API; BM-03 cross-OS gate is closed

## VER-BM04-GATE — BM-04 — 2026-07-19T19:10:30Z

- Source revision and dirty state: `b3b95618deca53f1938915aa879c10ced3f5ae15`; dirty only with BM-04 implementation, governance, and evidence
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; locked repository Python environment; `sysinfo` 0.39.6; `Cargo.lock` unchanged from BM-03
- Command: machine record `BM-04-1784488230746317100`; full universal/schema/governance gate through an external verifier copy
- Start/end/duration: machine-record Unix UTC nanoseconds `1784488230746317100` / `1784488261002477300` / 30.256976 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `40B5868AA502C4F1C7160BDA390518B90E41E6D16E436182CE242356BB702BDE`; stderr `457EBE66EA764F3312ADD3F835633F4CC0D538CC2CDE51F63DD1CA1EC146096A`
- Fixtures/manifests/bundle IDs: exact event/work loads, unavailable/unstable/error fail-closed fixtures, live CPU/allocation/I/O loads, observer-cost record, and complete existing repository corpus
- Counter availability and privileges: live unprivileged local process/filesystem counters with OS/cache qualifications; no physical-host, energy, or enforcement claim
- Result: pass
- Reviewer/attestation: every counter records expected/observed/error/resolution/coverage/qualification, non-ready evidence fails closed, observer cost is explicit, and strict workspace/Python/schema/governance gates passed locally; hosted closure remains attached to the focused commit

## VER-BM04-HOSTED — BM-04 — 2026-07-19T19:13:59Z

- Source revision and dirty state: `7cbf4547c5df0c5a7815e662537f8cbe906b21e0`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29700201610, attempt 1
- Start/end/duration: `2026-07-19T19:13:59Z` / `2026-07-19T19:18:49Z` / 290 s
- Exit code: all four jobs concluded `success`; job IDs 88227876798 (macOS arm64), 88227876802 (Linux), 88227876811 (macOS Intel), and 88227876816 (Windows)
- Stdout/stderr artifact hashes: retained by GitHub Actions
- Fixtures/manifests/bundle IDs: exact and live calibration workloads plus fail-closed coverage fixtures on every hosted runner
- Counter availability and privileges: live hosted unprivileged process/filesystem counters with OS/cache qualifications; no physical-host, energy, or enforcement claim
- Result: pass
- Reviewer/attestation: exact head SHA, workflow conclusion, and all four matrix-job conclusions inspected through the GitHub API; BM-04 cross-OS gate is closed

## VER-BQ01-GATE — BQ-01 — 2026-07-19T19:18:51Z

- Source revision and dirty state: `7cbf4547c5df0c5a7815e662537f8cbe906b21e0`; dirty only with BQ-01 implementation, governance, and evidence
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; locked repository Python environment; `Cargo.lock` SHA-256 `36E07DEA7070121EFDED59BA5D48FF1DAE12D3694E5550B188CE40D9CEAC20E4`
- Command: machine record `BQ-01-1784488731506570500`; full universal/schema/governance gate through an external verifier copy
- Start/end/duration: machine-record Unix UTC nanoseconds `1784488731506570500` / `1784488759997382000` / 28.4915773 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `242A6A260AAB1EDDFF078C008DEE7B8E576E0C5E9A6C6E3B97B8B12C0D12A5D3`; stderr `E5E4A269ED8A373B5FA7A5F346F788D2BA1BFB07E51AEBC82CBA984C4A08DB3D`
- Fixtures/manifests/bundle IDs: boundary-equality enumeration, nested scopes, overlapping rolling windows, exact expiry, unavailable measurement, overflow, and complete existing repository corpus
- Counter availability and privileges: deterministic arithmetic fixtures only; no OS enforcement or privileged counter claim
- Result: pass
- Reviewer/attestation: typed units, checked projections/commits, exact resets/windows, missingness, and strict workspace/Python/schema/governance gates passed locally; hosted closure remains attached to the focused commit

## VER-BQ01-HOSTED — BQ-01 — 2026-07-19T19:22:58Z

- Source revision and dirty state: `5cbd77e664f01a6f07f972e78967d5b6b05d6246`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29700476523, attempt 1
- Start/end/duration: `2026-07-19T19:22:58Z` / `2026-07-19T19:28:15Z` / 317 s
- Exit code: all four jobs concluded `success`; job IDs 88228587130 (macOS Intel), 88228587140 (Windows), 88228587147 (macOS arm64), and 88228587157 (Linux)
- Stdout/stderr artifact hashes: retained by GitHub Actions
- Fixtures/manifests/bundle IDs: typed arithmetic boundaries, nested scopes, overlapping rolling windows, expiry, missingness, and overflow on every hosted runner
- Counter availability and privileges: deterministic arithmetic fixtures only; no OS enforcement or privileged counter claim
- Result: pass
- Reviewer/attestation: exact head SHA, workflow conclusion, and all four matrix-job conclusions inspected through the GitHub API; BQ-01 cross-OS gate is closed

## VER-BQ02-GATE — BQ-02 — 2026-07-19

- Source revision and dirty state: `5cbd77e664f01a6f07f972e78967d5b6b05d6246`; dirty only with BQ-02 implementation, governance, and evidence
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; locked repository Python environment; `Cargo.lock` SHA-256 `5D5E304EA40DB8C267DCDF1F926959DF290BE0A10BBE1322341E4A18FDB6CA3E`
- Command: machine record `BQ-02-1784489349257426500`; full universal/schema/governance gate through an external verifier copy
- Start/end/duration: machine-record Unix UTC nanoseconds `1784489349257426500` / `1784489373149766000` / 23.8929798 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `13A9D41E2B6413E972A108CF81531387EB768A5465BD3AF27504527A52B28343`; stderr `2AEC63E277D8D5D34FABC101FE4452F7ACED96C2879B172F22D73525AE24E50A`
- Fixtures/manifests/bundle IDs: all four outcomes, 100 identical-input repetitions, missing hard-counter rejection, contradictory projections, and complete existing repository corpus
- Counter availability and privileges: deterministic projection fixtures only; no OS enforcement or privileged counter claim
- Result: pass
- Reviewer/attestation: immutable policy binding, stable outcome precedence/reason codes, canonical decision evidence, fail-closed projection validation, and strict workspace/Python/schema/governance gates passed locally; hosted closure remains attached to the focused commit

## VER-BQ02-HOSTED — BQ-02 — 2026-07-19T19:34:06Z

- Source revision and dirty state: `7fd5a9e4e5b0763c8eb5e60d436668a55db495a7`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29700864636, attempt 1
- Start/end/duration: `2026-07-19T19:34:06Z` / `2026-07-19T19:39:03Z` / 297 s to the final job completion
- Exit code: all four jobs concluded `success`; job IDs 88229576475 (macOS arm64), 88229576477 (macOS Intel), 88229576489 (Linux), and 88229576535 (Windows)
- Stdout/stderr artifact hashes: retained by GitHub Actions
- Fixtures/manifests/bundle IDs: all four decisions, deterministic evidence repetitions, missing hard-counter rejection, and complete existing corpus on every hosted runner
- Counter availability and privileges: deterministic projection fixtures only; no OS enforcement or privileged counter claim
- Result: pass
- Reviewer/attestation: exact job head SHAs and all four matrix-job conclusions inspected through the GitHub API; BQ-02 cross-OS gate is closed

## VER-BQ03-GATE — BQ-03 — 2026-07-19

- Source revision and dirty state: `7fd5a9e4e5b0763c8eb5e60d436668a55db495a7`; dirty only with BQ-03 implementation, governance, and evidence
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; locked repository Python environment; `Cargo.lock` unchanged from BQ-02
- Command: machine record `BQ-03-1784489842292267700`; full universal/schema/governance gate through an external verifier copy
- Start/end/duration: machine-record Unix UTC nanoseconds `1784489842292267700` / `1784489878418390200` / 36.1270949 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `F4C7199604F7907283B0F85EF0C5531F82AEED96C0DD6794EB98405F259032A5`; stderr `0EE17058BD7D36377238D615C8B0A23145EDF3DC4720E3CE5007A6A12DC4E7A2`
- Fixtures/manifests/bundle IDs: compliant, soft, hard, invalid-edge, terminal-reentry, every-transition fault injection, and complete existing repository corpus
- Counter availability and privileges: deterministic lifecycle fixtures only; no OS enforcement or privileged counter claim
- Result: pass
- Reviewer/attestation: explicit lifecycle edges, one terminal outcome, valid failure evidence, hard-violation ineligibility, no-resume recovery, and strict workspace/Python/schema/governance gates passed locally; hosted closure remains attached to the focused commit

## VER-BQ03-HOSTED — BQ-03 — 2026-07-19T19:41:33Z

- Source revision and dirty state: `64c1f6d6d992f67859da8189e4c77132d65cf34c`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29701098985, attempt 1
- Start/end/duration: `2026-07-19T19:41:33Z` / `2026-07-19T19:45:50Z` / 257 s
- Exit code: all four jobs concluded `success`; job IDs 88230181740 (Linux), 88230181751 (macOS arm64), 88230181755 (macOS Intel), and 88230181760 (Windows)
- Stdout/stderr artifact hashes: retained by GitHub Actions
- Fixtures/manifests/bundle IDs: compliant/soft/hard paths, invalid transitions, every-transition recovery, and complete existing corpus on every hosted runner
- Counter availability and privileges: deterministic lifecycle fixtures only; no OS enforcement or privileged counter claim
- Result: pass
- Reviewer/attestation: exact run head SHA, run conclusion, and all four matrix-job conclusions inspected through public GitHub metadata; BQ-03 cross-OS gate is closed

## VER-BQ04-GATE — BQ-04 — 2026-07-19

- Source revision and dirty state: `64c1f6d6d992f67859da8189e4c77132d65cf34c`; dirty only with BQ-04 implementation, governance, and evidence
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; locked repository Python environment; `Cargo.lock` unchanged from BQ-03
- Command: machine record `BQ-04-1784490255254044900`; full universal/schema/governance gate through an external verifier copy
- Start/end/duration: machine-record Unix UTC nanoseconds `1784490255254044900` / `1784490292513148300` / 37.2601049 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `EC2D2F035876CEF9C5F2AC90A0494DE576FBD5EC85A257E3A340B06FFAEB48F2`; stderr `372D496CADBCACBCF88F59630FB68D36870DACDD41FF92C568BAF25B4FC0B423`
- Fixtures/manifests/bundle IDs: two-step under-budget run, preflight work denial, missing-counter denial, five post-step overages, retained prior evidence, and complete existing repository corpus
- Counter availability and privileges: deterministic portable supervisor fixtures only; no OS hard-cap or privileged counter claim
- Result: pass
- Reviewer/attestation: all five required counters, no-call preflight rejection, checked lifetime accounting, post-step termination, evidence preservation, budget-eligibility qualification, and strict workspace/Python/schema/governance gates passed locally; hosted closure remains attached to the focused commit

## VER-BQ04-HOSTED — BQ-04 — 2026-07-19T19:49:14Z

- Source revision and dirty state: `e9dbb0e8907c7928c08faab9f39be802fd632af8`; clean pushed revision
- OS/architecture/physical-or-CI: Windows/x86_64, Linux/x86_64, macOS/arm64, macOS/x86_64; GitHub hosted CI; ephemeral virtual machines
- Toolchain/dependency-lock hashes: workflow-pinned Rust 1.96.0, Python 3.12, and uv 0.11.29; source locks at the recorded revision
- Command: GitHub Actions `BONSAI baseline` push run 29701331312, attempt 1
- Start/end/duration: `2026-07-19T19:49:14Z` / `2026-07-19T19:54:00Z` / 286 s
- Exit code: all four jobs concluded `success`; job IDs 88230772753 (Windows), 88230772760 (macOS Intel), 88230772774 (macOS arm64), and 88230772789 (Linux)
- Stdout/stderr artifact hashes: retained by GitHub Actions
- Fixtures/manifests/bundle IDs: under-budget primitive loop, unavailable/preflight no-call rejection, five intentional overages, prior evidence preservation, and complete existing corpus on every hosted runner
- Counter availability and privileges: deterministic portable supervisor fixtures only; no OS hard-cap or privileged counter claim
- Result: pass
- Reviewer/attestation: exact run head SHA, run conclusion, and all four matrix-job conclusions inspected through public GitHub metadata; BQ-04 cross-OS gate is closed

## VER-BK01-GATE — BK-01 — 2026-07-19

- Source revision and dirty state: `e9dbb0e8907c7928c08faab9f39be802fd632af8`; dirty only with BK-01 implementation, governance, and evidence
- OS/architecture/physical-or-CI: Windows/x86_64; local; physical/virtual status unknown
- Toolchain/dependency-lock hashes: Rust 1.96.0; locked repository Python environment; `Cargo.lock` SHA-256 `6C703B765B3AE1CFF02B2FB34385A194F7F52E2629D8034BC9028803C599B736`
- Command: machine record `BK-01-1784490756845337800`; full universal/schema/governance gate through an external verifier copy
- Start/end/duration: machine-record Unix UTC nanoseconds `1784490756845337800` / `1784490789599125200` / 32.7546683 s
- Exit code: 0
- Stdout/stderr artifact hashes: stdout `E34E712263E18EDC809B10A410B7C24D0ED3BAC25E932BED2F33DF81619BDC07`; stderr `5A0489F49A722C6976B0B2B3E387CE6E2EAF324875C597EB27BE3DB8FA16E2CE`
- Fixtures/manifests/bundle IDs: version-pinned reward-rate golden, 100 deterministic derivations, dependency cycle, wrong version, missing source, and complete existing repository corpus
- Counter availability and privileges: deterministic analytical fixtures only; no platform or privileged counter claim
- Result: pass
- Reviewer/attestation: complete registry metadata, dependency DAG, cycle/version rejection, rational golden output, missingness propagation, and strict workspace/Python/schema/governance gates passed locally; hosted closure remains attached to the focused commit
