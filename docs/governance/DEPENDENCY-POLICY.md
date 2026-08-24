# BONSAI dependency and supply-chain policy

Status: BV-13 authority  
Owner: reproducibility lead  
Decisions: D-07, D-08 / [ADR 0004](../architecture/adr/0004-packaging-and-offline-reproducibility.md)

## Pins

- Rust workspace: committed `Cargo.lock`, toolchain `rust-toolchain.toml` (1.96.0).
- Python package: committed `uv.lock`, `requires-python = ">=3.12,<3.15"`.
- CI action versions are pinned by full commit SHA in `.github/workflows/ci.yml`.
- Codegen (Protobuf/schema) versions are the locked crate and `protobuf==7.35.1` pin.

Unlocked or floating restoration is rejected. A rebuild that needs the public network after the lockfiles and vendor/cache are present is a gate failure.

## SBOM and archives

`scripts/generate_sbom.py` emits CycloneDX-shaped JSON from the committed lockfiles into `evidence/release-candidate/`. `scripts/package_rc.py` records source/native/Python package file lists and SHA-256 hashes. `scripts/offline_restore.py` vendors or uses the existing cache and rebuilds with network disabled.

## Advisories and waivers

Critical or high advisories that affect required targets must be fixed, or recorded in `fixtures/supply-chain/v1/waivers.json` with owner, expiry, and residual risk. An empty waiver file means **no silent critical waiver**. A missing field is a gate failure. Severity `critical` without an explicit `accepted_by` and `expires_on` fails closed.

## Offline and clean-machine evidence

| Evidence | What it proves | What it does not prove |
|---|---|---|
| `cargo test --offline` / `uv run --frozen` after a populated cache | Required targets rebuild without further network | A blank physical host or isolated VM |
| Vendor/offline scripts | An operator can produce a vendor tree from locks | Hosted CI is that physical restore |
| Physical clean-machine restore | D-08 acceptance on a named host | Not-run unless a host attestation says otherwise |

This cycle records **CI-scale offline rebuild** as the available gate and leaves physical clean-machine restore **not-run**. Do not treat one hosted job as a clean-machine attestation.

## Distribution

Native CLI binaries, Python source/wheels, and the complete source repository are the D-07 forms. OCI images are conveniences only. crates.io, PyPI, git tags, and marketing claims remain unauthorized (OD-03 / BV-16).
