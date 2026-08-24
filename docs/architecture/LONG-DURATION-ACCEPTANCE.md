# Long-duration physical-host acceptance

Status: BV-14 harness  
Profile: D-16 Long (L)

L requires at least 72 hours, 10 million steps, and three paired seeds on each required physical OS under documented thermal and operational conditions.

## This cycle (OD-01)

The harness exists:

- [`fixtures/l-profile/v1/manifest.json`](../../fixtures/l-profile/v1/manifest.json)
- [`fixtures/l-profile/v1/resource-policy.json`](../../fixtures/l-profile/v1/resource-policy.json)
- host attestations under [`fixtures/host-attestation/v1/`](../../fixtures/host-attestation/v1/)
- a CI-scale duration probe in `bonsai-accept` (`32` steps, `hosted-ci`, `long_duration_claim=false`)

Windows, macOS, and Linux physical 72-hour hosts were **not available**. Attestations are `not-run` / `indeterminate`. A short CI probe **cannot** be promoted to L acceptance.

Failure and recovery evidence for the harness is the existing BR-05 supervisor recovery path, exercised by the M4 suite after an interrupted `running` state.

Do not update [TEST-MATRIX](../verification/TEST-MATRIX.md) L rows to pass from this document.
