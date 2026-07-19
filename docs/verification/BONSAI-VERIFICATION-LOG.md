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
