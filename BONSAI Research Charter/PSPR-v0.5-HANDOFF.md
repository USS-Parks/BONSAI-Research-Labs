# PSPR v0.5 Project Handoff

Date: 2026-09-05 (America/Los_Angeles)
Status: **APPROVED — FULL BX-01–BX-40 STS AUTHORIZED; BX-01–BX-04 VERIFIED; BX-05 EXECUTING**

## Current execution authority — 2026-09-05

> Approved for STS now. Authorization to run this until verified complete, committing and merging with main after each prompt is finished fully, and all 40 prompts are successfully executed.

The approved project plan section 14 records this authority. Begin/resume BX-01 and continue sequentially after each prescribed gate and commit/main publication. The sections below preserve the earlier import snapshot; their pending-approval statements are historical.

## Review document and authority

[PSPR v0.5-draft.1](./BONSAI-PSPR-v0.5-EXECUTABLE-RESEARCH-AND-GOVERNED-EVOLUTION.md) is the single project-local review copy. It was imported byte-for-byte from the user-supplied attachment; SHA-256: `af7d66762d3d77569a8f47e774ff9924cd31a474c366dc45b02878170fe19f7e`.

Importing the draft into its proposed repository location is document integration only. It does not adopt the plan as approved, supersede the approved v0.1–v0.4 history, start BX-01, or authorize source publication. All BX-01–BX-40 remain NOT STARTED. No approval or STS wording accompanied the attachment.

The draft's references to its original outputs folder describe its source-session history. Use the linked project copy for review here. The companion architecture review and original transfer note named in section 13 were not included in the supplied attachment; this handoff does not recreate or claim to verify them. Their supporting review evidence must be supplied or independently re-established during authorized BX-01.

## Reconciliation at import

- Checked local main: `2f35355c6d12d019eb8625cb3bd38728d90ee029`, matching the cached origin/main. The working tree was clean before document integration.
- The earlier user-authorized commit/merge task fast-forwarded this checkout to that SHA. The draft's older local HEAD and behind counts in sections 0.3 and 8 are preserved historical observations, not current checkout status. No checkout update was performed for this import.
- The [canonical PSPR](./BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md) remains the parent governance record. [M4](./BONSAI-PSPR-v0.4-M4-ACCEPTANCE-AND-RELEASE.md), the [DEVLOG](../docs/sessions/BONSAI-DEVLOG.md), and the [verification log](../docs/verification/BONSAI-VERIFICATION-LOG.md) retain their recorded implementation history and physical-host exceptions. This import does not revalidate their scientific or live-system claims.
- The canonical checkout remains the active lane. The historical `C:/tmp/bonsai-m0-sts` worktree remains at `eaa0e52ec5a6dc78ab1a360f2a11c2201c7a5e9d`; the preceding merge task verified it clean with no unpublished commits relative to main and approximately 1.36 GiB of target/virtual-environment data. It is retained pending explicit removal authorization. Canonical generated-data size remains partially unreadable; the draft's 8.15 GiB lower bound is historical, not a fresh measurement.
- The inherited AGENTS.md references RTK.md, but `C:/Users/17076/RTK.md` was absent at import. No RTK-specific instructions were inferred.

## Next step

Review v0.5 and record any requested changes. The draft recommends M5a (BX-01–BX-06) as the first usable cut. Plan approval alone does not start execution. After explicit approval and scoped STS authorization, BX-01 is first and must revalidate the baseline, review findings, full applicable gates, hosts, and resource availability.

Use the existing DEVLOG and verification log. Preserve one prompt per focused commit, the M5a–M5f dependency order, all gates, and the publication/host boundaries in the draft. No new worktree, experiment, implementation commit, or push is part of this import.

## Current continuation — BX-02

BX-01 closed at `51637ee2dfb3b21673bfeddecb8533a8dd6e29d5`, hosted run 33998986034 fully green. BX-02 is executing. User-local WSL Rust 1.96.0 is installed without sudo/profile changes; the three existing process_transport tests pass on x86_64-unknown-linux-gnu. Reuse the canonical target root with the Linux target triple and limit build jobs to four. Windows .venv remains intact; WSL subprocess fixtures use system Python.

## Current continuation — BX-03

BX-02 closed at `9a2457afd873bac167ffd61602b64f624f7ea089`, hosted run 33999964570 fully green. BX-03 is executing against the existing unprivileged WSL2 systemd delegation. The supervisor subgroup prerequisite was verified; no global controller changes or physical-host claim.

## Current continuation — BX-04

BX-03 closed at `1cd14055d32bf448fb844fba73034442093890c0`, hosted run 34000956057 fully green. BX-04 now extends the existing ScenarioSpec and Protobuf payload boundary. Preserve the archived semantic_stream hashes and protocol epoch.

## Current continuation — BX-05

BX-04 closed at corrected main `9c15015a5c6252c5c461f1e13b6be154e48d1c9a`, hosted run 34002423080 fully green. BX-05 now composes the existing Rust runtime/governor/ingestor/bundle/report and Python PrimitiveTabularControl with the causal Protobuf environment. Use target/linux-venv through UV_PROJECT_ENVIRONMENT for Linux; preserve Windows .venv. Initial BX-04 d3b2087 was corrected solely for generated-file checkout bytes.


### BX-05 implementation checkpoint — 2026-09-06T01:27:15.799Z

BX-05 remains executing and uncommitted. Main/origin main remain `9c15015a5c6252c5c461f1e13b6be154e48d1c9a`; BX-01 through BX-04 are complete.

Implemented gated transport attachment using the verified LinuxAuthority, shared ordered adapter lifecycle, a real PrimitiveTabularControl child with typed reward/accounting payloads, and an initial `cargo xtask run` composition. The runner exchanges real causal actions, admits and records acting/learning work separately, measures cgroup CPU/memory/tasks plus actual `/proc` RSS and agent storage, appends bounded validated events, and generates reports after execution. Source/component identity verification was added after the successful diagnostic and still needs live verification.

Checks so far: Windows runtime compile passed; Linux runtime/initial runner compile passed; Python existing 57 tests passed after lifecycle extraction; live primitive learner plus environment suite 15 tests passed; strict Python typing and Ruff passed at their recorded intermediate revisions. The live delegated transport probe proved before-exec membership, preserved arguments/cwd, and empty cleanup. These are intermediate checks, not the final BX-05 gate.

Diagnostic outputs preserved under `target/bx05-diagnostic-01` (20 real steps, 320 events), `target/bx05-s-initial-01` (INCOMPLETE action deadline, 16,979 events), and `target/bx05-s-initial-02` (2,000 steps, 966 completed episodes, reward sum 957, 8,000 work items, 35,822 events, 16,812,886 event bytes, execution 33,745,350,715 ns). The first S attempt exposed observer persistence inside the pre-dispatch timer. The corrected timing keeps the 50 ms dispatch-to-validated-response limit unchanged and includes persistence in the overall run wall budget. Both attempts must remain honestly distinguished.

Remaining before BX-05 completion: refactor the long runner methods into focused helpers to pass strict Clippy (do not silence the lints); finish canonical result-bundle roles/manifests and complete source/input identity retention; make finalization/storage failures leave unequivocal INCOMPLETE status; enforce total observer-output and final wall budgets; perform source-change and manifest-negative checks; execute classified forced agent/resource/storage/cancellation cases and real delayed-action rejection; rerun final S and the full repository gates; refresh Cargo.lock-dependent SBOM/RC checksums; archive byte-verified evidence; commit/push main and wait for required hosted matrix. C0/C1 adjudication stays BX-06. No new worktrees created.


BX-05 checkpoint 2026-09-06T01:41:26.058Z: strict Linux Clippy and all 59 Python tests pass. Real child exit, storage quota, late response, in-flight cancellation, and initialization-time kernel OOM now have expected INCOMPLETE classifications; detailed artifact paths and failed-attempt history are in the latest BX-05 DEVLOG checkpoint. BX-05 is still uncommitted: canonical bundle packaging, finalization/output/wall enforcement, final frozen-source S/failure verification, and publication gates remain. No active processes remain.


BX-05 checkpoint 2026-09-06T01:48:49.662Z: durable INCOMPLETE finalization markers and accepted-event partial reward totals are implemented and live-verified against a report-write failure. Actual compiler identity and Linux inventory packaging compile under strict Clippy; live/schema validation remains. Canonical bundle-role assembly, total quota/wall enforcement, final acceptance, and publication are still pending. See latest DEVLOG details; BX-05 is not committed or complete. No processes remain active.

### BX-05 local acceptance — 2026-09-06 UTC

All local BX-05 acceptance gates passed: full S in 56.327551905 seconds; all 11 live cases; independent event/resource/failure reconciliation; five preflight negatives; 212 Windows / 214 Linux Rust tests and 59 Python tests on both. The exact source/result archive is `evidence/verification/bx-05/final-live.zip` (SHA-256 `315d6c94069b1b23fd7ebd2e524ad5d00053a3dfc9c638ef100f3597ad8e4585`). See the latest DEVLOG and verification entry for evidence and corrected failed attempts.

Still required: final documentation and staged-byte checks, focused BX-05 commit/push to main, then all required hosted jobs. Do not start BX-06 before publication passes. Keep C0/C1 and physical-host claims unadjudicated. Canonical generated data is at least 20.28 GiB; the inactive clean historical worktree retains 1.36 GiB pending removal authorization. No new worktree was created.
