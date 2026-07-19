# BONSAI publication policy

Status: accepted  
Effective: 2026-07-18  
Owner: BONSAI maintainers  
Decision: D-09 / ADR 0001

## Current source-repository exception

The [2026-07-18 public-repository addendum](addenda/2026-07-18-public-repository-target.md) authorizes the charter history and gated M0 source/documents to be published to `USS-Parks/BONSAI-Research-Labs` branch `main` after a clean secret/privacy scan. The exception is target- and artifact-specific; it does not authorize later milestones, releases, packages, evidence bundles, datasets, or other destinations.

## Default state for all other publication

Except for the approved public source-repository scope above, BONSAI artifacts remain local/private through the final evidence audit. Local development and local commits are permitted only within authorized STS prompts. None of the following is implied by plan approval, execution authorization, a passing gate, a local commit, a CI run, or instrument completion:

- changing repository visibility;
- pushing to a new or public remote;
- uploading packages, releases, images, datasets, reports, or evidence;
- submitting an external benchmark;
- announcing scientific or capability claims;
- publishing on behalf of Oak Lab or any other party.

Each external publication action requires explicit user authorization naming the target and artifact set.

## Mandatory pre-publication review

Before requesting or performing publication, record a review that verifies:

1. The exact clean source revision, artifact hashes, dependency locks, SBOMs, and build provenance agree.
2. The release/evidence audit and applicable PSPR gates passed; failures and indeterminate verdicts remain visible.
3. Claims do not exceed the machine-generated C0–C5 evidence and do not represent BRDC-1 as an unpublished Oak Lab implementation.
4. Repository and artifact license metadata says `MIT OR Apache-2.0`, and third-party notices/provenance are complete.
5. A secret and privacy scan covers source, history in scope, logs, bundles, reports, CI artifacts, and package metadata.
6. Redaction review removes credentials, tokens, user-identifying paths, hostnames, device serials, raw environment dumps, private source material, and unauthorized third-party data.
7. Physical-host, hosted-CI, estimated, unavailable, E0–E3, and simulated evidence remain clearly distinguished.
8. The destination, visibility, audience, retention, and rollback or takedown owner are recorded.
9. The user gives explicit publication authorization after reviewing the above record.

A failed, incomplete, or stale review blocks publication. Silence, prior remote existence, or authorization for a different target is not approval.

## Contribution boundary

External contributions are accepted only after license, provenance, privacy, and authorization review. Private access does not grant permission to redistribute source or evidence.

## Incident response

If unauthorized publication or secret exposure is detected, stop further transmission, preserve a minimal audit record, notify the owner through an authorized channel, rotate affected credentials outside the repository workflow, and record affected artifacts and remediation. Do not erase Git or evidence history to conceal the event.
