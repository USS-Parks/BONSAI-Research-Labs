# BONSAI release-candidate notes 0.0.0

Status: **release-candidate artifacts only**  
Date: 2026-08-24  
Authority: PSPR v0.4 / OD-03

This directory records source, native-workspace, and Python package identities plus SBOMs and hashes. It is not a public release.

## OD-03 non-actions

- NO git tag
- NO crates.io upload
- NO pypi upload
- NO marketing claim

Publication remains separately unauthorized.

## What is included

- Locked Rust and Python workspaces (`Cargo.lock`, `uv.lock`)
- CycloneDX-shaped SBOMs under `evidence/release-candidate/`
- SHA-256 sums of policy, threat model, L harness, and SBOM files
- Source package file list (not a uploaded archive)
- Independent verification of the committed BC-12 valid bundle fixture

## What is not included

- A 72-hour physical-host L pass (Windows, macOS, and Linux attestations are **not-run**)
- Instrument completion
- C4/C5 pass
- OaK / Oak Lab reproduction
- Privileged collector qualification

Native crate packages and Python sdists may be produced locally with `cargo package --list` and `uv build` for inspection. Those commands are not registry uploads.
