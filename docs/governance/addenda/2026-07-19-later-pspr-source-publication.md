# PSPR addendum: continuing source publication

- Status: approved by direct user instruction
- Approved: 2026-07-19 America/Los_Angeles / 2026-07-19 UTC
- Owner: repository owner
- Supersedes: the M0-only source-publication scope in the 2026-07-18 public-repository addendum
- Does not supersede: prompt gates, redaction/secret review, scientific claim boundaries, release audit, or separate authority for non-source publication

## User instruction

During BC-08 STS, the user stated: `Later milestone publication is authorized.`

## Authorized scope

After its prescribed gate passes and its focused commit exists, each later PSPR source prompt may be pushed by fast-forward to public `USS-Parks/BONSAI-Research-Labs` branch `main`. This includes source, schemas, tests, governance documents, DEVLOG/verification indexes, and small redacted prompt evidence committed under the PSPR.

The authorization applies prompt by prompt; it does not waive a failed gate or authorize force-push, credentials, secrets, personal paths, unredacted host evidence, packages, releases, images, datasets, external benchmark submissions, scientific announcements, or claims beyond machine evidence.

## Required publication gate

Before each push, verify the focused commit, clean tracked state, full repository gate, no-slop/secret/privacy controls, authorized remote/branch, and fast-forward relationship. After push, record the remote SHA and exact hosted-CI outcome when hosted evidence is part of the prompt closeout.
