# PSPR addendum: public BONSAI Research Labs repository

- Status: approved by direct user instruction
- Approved: 2026-07-18 America/Los_Angeles / 2026-07-19 UTC
- Owner: repository owner
- Supersedes: D-01 repository URL and the private-visibility portion of D-09
- Does not supersede: dual licensing, scientific gates, prompt order, claim boundaries, secret/privacy review, or separate authorization for later releases and evidence publication

## User instruction

The user designated `https://github.com/USS-Parks/BONSAI-Research-Labs` as the new BONSAI repository and instructed the STS session to commit and push all existing work and documents to its `main`, with no secrets.

## Adjudication

The authoritative GitHub repository is now `USS-Parks/BONSAI-Research-Labs`. It is a public repository whose default branch is `main`. The prior `USS-Parks/BONSAI` remote remains historical and is no longer an authorized publication target.

This instruction explicitly authorizes one publication scope:

- the complete approved charter-package history;
- root governance and license documents;
- M0 source, tests, CI configuration, development logs, and verification records that pass redaction review;
- the focused M0 commits produced by BG-01 through BG-10;
- push of the gated result to `USS-Parks/BONSAI-Research-Labs` branch `main`.

It does not authorize credentials, secrets, personal paths, raw environment dumps, host identifiers, unredacted machine evidence, future milestone code, packages, releases, images, datasets, external benchmark submissions, or scientific/capability claims.

## Consequences

1. Repository metadata and current documentation use the new URL.
2. The prior private-through-audit language remains in PSPR v0.1 and ADR 0001 as historical text, but this addendum supersedes it for the scoped public source repository.
3. A secret/privacy scan and source-of-truth audit are mandatory immediately before push.
4. Future external artifacts or targets still require explicit authorization and the publication-policy review.
5. If remote `main` is not empty or cannot accept a fast-forward history, execution stops rather than overwriting it.

## Acceptance evidence

Before publication, verify via GitHub repository metadata that the target owner/name, public visibility, and default branch match this addendum; verify the remote branch state; run M0 gates; scan tracked content and pending history for secret-like material and user-identifying paths; record the clean source revision; then push without force.
