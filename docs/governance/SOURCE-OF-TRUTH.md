# BONSAI source-of-truth governance

Status: accepted  
Effective: 2026-07-18  
Owner: BONSAI maintainers  
Applies to: all BONSAI planning, implementation, evidence, and publication work

## Authority hierarchy

When sources conflict, use this order:

1. The user's current instruction and any explicitly approved PSPR addendum.
2. The [BONSAI Research Charter](../../BONSAI%20Research%20Charter/BONSAI-RESEARCH-CHARTER.md) for scientific intent, scope, exclusions, measurement obligations, and claim boundaries.
3. The [OaK Evidence and Traceability Register](../../BONSAI%20Research%20Charter/OAK-EVIDENCE-AND-TRACEABILITY.md) for the boundary between public OaK claims, primary-source support, and BONSAI inference.
4. The [approved PSPR](../../BONSAI%20Research%20Charter/BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md) for implementation order, prompt scope, gates, and settled defaults.
5. Approved architecture decision records, versioned schemas, and metric specifications.
6. The DEVLOG and verification log for executed history and evidence identity.
7. Code, tests, generated documentation, and result bundles at the recorded source revision.

Lower-ranked material cannot silently weaken, override, or reinterpret a higher-ranked source.

## Authorization boundary

Planning, drafting, approval, status editing, and repository existence are not execution authorization. Execution begins only when the user says `run it STS`, authorizes a milestone such as `run M0 STS`, or explicitly authorizes named prompt IDs.

Authorization is bounded by the named prompts and their dependencies. It does not automatically authorize:

- later milestones or prompts;
- public visibility or external publication;
- remote creation or release/package upload;
- credentials, secrets, privileged collectors, or external services;
- destructive actions or revived parked scope.

When work reaches the end of its authorized prompt set, stop at the approval gate. M0 authorization ended after BG-10 and did not itself authorize BC-01. On 2026-07-18 the user then instructed `Continue to STS`; the current expanded scope and prompt status are recorded in the PSPR execution history and DEVLOG.

## Prompt status semantics

- `[ ]` — not started.
- `[~]` — executing but not gated.
- `[x]` — the prescribed gate passed, the execution record is complete, and the focused commit exists.
- `[!]` — blocked, with the blocker and evidence recorded.
- `[-]` — superseded by a named and dated approved addendum.

A prompt is not complete merely because files exist. A prompt may become `[x]` only after its gate passes. One prompt normally maps to one focused commit; any inseparable bundle must be justified in the DEVLOG.

## Verification and evidence

The universal verification gate in the PSPR applies after its commands exist. Before then, the prompt's bootstrap gate is authoritative. Verification records must include the exact command, UTC start and end, exit code, source revision and dirty state, sanitized platform fingerprint, and hashes of retained output artifacts.

Mock-only evidence cannot close resource enforcement, process isolation, replay isolation, energy or hardware-counter collection, tamper detection, or physical-platform behavior. Missing evidence stays unavailable or indeterminate; it never becomes zero or pass.

## History and addenda

Approved source history is append-only in meaning. Do not silently rewrite a settled decision, completed prompt, failed result, or evidence identity.

A change that affects dependencies, acceptance logic, scientific meaning, trust boundaries, resource policy, claim eligibility, or parked scope requires a dated, reviewable PSPR addendum. The addendum must identify:

- the exact source and decision it supersedes;
- the reason and owner;
- affected prompts, schemas, metrics, evidence, and releases;
- which prior evidence becomes stale or remains valid;
- the new acceptance gate;
- explicit user approval.

Ordinary corrections that do not alter meaning may use normal review, but must retain honest Git history.

## Execution records

Once BG-06 establishes the machinery, execution is recorded in:

- `docs/sessions/BONSAI-DEVLOG.md`;
- `docs/verification/BONSAI-VERIFICATION-LOG.md`;
- `docs/governance/RISK-AND-BLOCKER-REGISTER.md`;
- `docs/governance/PARKED-SCOPE-LEDGER.md`;
- `docs/governance/CLAIM-TO-EVIDENCE-MATRIX.md`.

The repository, logs, source revision, gate results, and publication state must agree. A local commit is not a pushed commit, a hosted CI result is not physical-host evidence, and instrument completion is not an evaluated-agent capability claim.
