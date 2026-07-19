# BONSAI Fresh-Session STS Handoff

Date: 2026-07-18  
Status: Charter and PSPR v0.1 approved; implementation not started; STS execution not yet authorized  
Authoritative repository: `https://github.com/USS-Parks/BONSAI`  
Authoritative local root: `C:\Users\17076\Documents\Reinforcement Learning Project`  
Authoritative branch: `main`  

## Read first

1. `BONSAI Research Charter/BONSAI-RESEARCH-CHARTER.md`
2. `BONSAI Research Charter/OAK-EVIDENCE-AND-TRACEABILITY.md`
3. `BONSAI Research Charter/BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md`
4. `BONSAI Research Charter/PSPR-HANDOFF.md`
5. This handoff.

The user approved all charter-package documents and the granular PSPR in their entirety on 2026-07-18. That approval settles D-01 through D-21 for PSPR v0.1. It does not itself authorize execution.

## Published starting state

- The documentation package is committed and pushed to the private `USS-Parks/BONSAI` repository on `main`.
- The repository was created during pre-STS closeout because no BONSAI remote previously existed.
- No BONSAI runtime, schema, adapter, measurement backend, governor, metric engine, reference agent, scenario, test suite, release artifact, or external publication has been implemented.
- All 104 PSPR prompt checkboxes remain unexecuted.
- Repository creation partially establishes the identity contemplated by BG-01, but BG-01 is not complete. The STS session must reconcile the pre-existing repository, create the prescribed root governance material, run the gate, log evidence, and only then mark it complete.

## Fresh-session start procedure

1. Verify the repository root, `main`, `origin`, remote SHA, and clean working tree. Do not allow Git to resolve to any parent repository.
2. Reconcile this handoff against the approved PSPR and current repository state.
3. Confirm the user has said `run it STS`, `run M0 STS`, or explicitly authorized named prompt IDs.
4. Create or use the required isolated worktree for the authorized STS session; concurrent sessions must not share a Git index.
5. Start at BG-01, treating the already-created private repository as pre-existing evidence to verify—not as a completed prompt.
6. Use one focused commit per prompt, maintain the DEVLOG and verification log once BG-06 creates them, and do not mark any prompt complete until its gate passes.

Suggested fresh-session instruction:

> Read `BONSAI Research Charter/STS-SESSION-HANDOFF.md`, reconcile it with the approved BONSAI PSPR and repository state, then run M0 STS from BG-01 in dependency order.

## Hard boundaries carried forward

- BONSAI remains the independent observer/governor, not an unpublished Oak Lab implementation.
- Track A remains strict single-pass, batch-size-one, and no-replay.
- Windows, macOS, and Linux remain first-class targets with CI evidence separated from physical-host evidence.
- Missing measurements remain unavailable, never zero.
- External publication, public visibility, privileged collector installation, credentials, and destructive actions require their own explicit authority.
- The repository remains private through the final evidence audit unless the user explicitly changes D-09 through an approved addendum.

