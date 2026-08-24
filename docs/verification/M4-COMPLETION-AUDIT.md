# M4 completion audit

Status: BV-16 evidence audit  
Date: 2026-08-24  
PSPR: v0.4 approved + full M4 STS

## Prompt close-out

| ID | Implementation | Gate | Honesty |
|---|---|---|---|
| BV-11 | `docs/security/THREAT-MODEL.md` + `fixtures/threat-model/v1/boundaries.json` | every TB-01–TB-10 row has controls, tests, residual risk, prompts; D-21 disclaimer is prominent | no sandbox claim |
| BV-12 | `crates/bonsai-accept` signing/redaction/bounds + ingest/storage/recovery suite | fuzz/property/adversarial tests; tamper/flood emit bounded failure | no external KMS |
| BV-13 | dependency policy, SBOM scripts, offline restore | lockfiles hashed; waivers empty; CI-scale offline rebuild | physical clean-machine **not-run** |
| BV-14 | L manifest, L policy, three host attestations, CI duration probe | D-16 minima declared; 72 h evidence **not-run / indeterminate** | OD-01: no invented pass |
| BV-15 | `docs/operator/*` + README | M1 heartbeat path; limitations restated | no evaluated-cycle pass |
| BV-16 | this audit, RC notes, hashes, SBOMs | independent bundle verify of committed valid fixture | NO tag/upload/marketing |

## Charter reconciliation

- Risks: R-13 accepted as honest host gap this cycle; R-08/R-09/R-14/R-15/R-16 remain active.
- Parked: P-01–P-09 unchanged. Hostile sandbox, publication, and OaK reproduction stay parked.
- Claims: C0–C5 and INSTRUMENT remain not-run / not a trophy pass. C4/C5 rules stay executable without a pass.
- Platforms: hosted four-OS baseline remains the portable gate. Physical Job Object / delegated cgroup / Apple energy stay unsupported or deferred as previously recorded.
- Long runs: L harness exists. Required Windows, macOS, and Linux **physical** 72 h / 10 M step / 3 paired-seed evidence is **not-run**.
- Offline restore: scripts + CI-scale `--offline` / `--frozen`. Clean physical-machine restore is **not-run**.
- Docs: operator handoff reproduces M1 and states non-claims.

## Instrument completion

Instrument completion criteria in the parent roster §12.2 are **not met**. Missing long-duration physical-host gates and physical clean-machine restore keep INSTRUMENT `not-run`. This audit records that fact. It does not convert not-run into pass.

## Independent verification

`cargo xtask bundle-check` / `validate_result_bundle` on `fixtures/bundle-validation/v1/manifest.json` must remain `VALID`. The tampered fixture must remain `INVALID`.
