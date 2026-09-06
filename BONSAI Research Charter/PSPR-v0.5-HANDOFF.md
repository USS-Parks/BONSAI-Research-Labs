# PSPR v0.5 Project Handoff

Date: 2026-09-05 (America/Los_Angeles)
Status: **APPROVED — FULL BX-01–BX-40 STS AUTHORIZED; BX-01–BX-10 VERIFIED; BX-11 EXECUTING**

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

### Current continuation — BX-06

BX-05 closed at `fb8ebfb0c5d903e0922726f8882ec5f58303abbc`; hosted run 34006775025 passed all four platform jobs and M1 semantic equivalence. Source/result archive and byte checks are recorded above. BX-06 is executing at the existing bundle-validator / bonsai-claims seam. Preserve the pure fixture adjudication and add an immutable facts boundary driven by real bundle bytes. No new worktree is needed.

### BX-06 local acceptance checkpoint

Implementation, portable tests, fresh S/smoke verification, twelve negative boundary cases, and exact-source archive passed. Next: final closeout checks, focused BX-06 main publication, and required hosted CI; then BX-07. BX-06 remains executing and M5a remains open pending publication. See the latest DEVLOG and verification entries for immutable receipts and records.

### BX-06 publication closeout / M5a complete / BX-07 start

Published main `911e7e35bc1c1d2dd90dd6c00ef1f08449cdf13a` matches remote main. Hosted [run 34008881505](https://github.com/USS-Parks/BONSAI-Research-Labs/actions/runs/34008881505) passed Windows, Linux, macOS Intel/arm64, and M1 semantic equivalence. The committed tree preserved all 76 files, 18 captured outputs, and the exact archive SHA with zero byte mismatches. BX-01 through BX-06 and M5a are complete. BX-07 now executes incremental lineage validation under the existing full-roster STS authorization; BX-08 through BX-40 remain unstarted.

### BX-07 local acceptance — publication pending

Incremental admission, rejection atomicity, 2,880 differential attempts and 72
scaling trials passed. Full Windows and WSL2 Linux gates passed (226/228 Rust
tests and 59 Python tests each). Exact source/trial archive and records are in
the latest DEVLOG and verification entries. BX-07 stays executing until its
focused main publication and all required hosted jobs pass. BX-08 remains unstarted.

### BX-07 publication closeout / BX-08 start

Published main `98cfd7845ad6068665f6aac0139c310107a794b2` matches remote main;
the checkout was clean. Hosted [run 34010216558](https://github.com/USS-Parks/BONSAI-Research-Labs/actions/runs/34010216558)
passed all four platform jobs and M1 semantic equivalence. All 36 committed
files, 12 captured outputs and the exact archive SHA passed byte verification.
BX-07 is complete. BX-08 now executes bounded persistent evidence and recovery
under the existing full-roster STS authority. BX-09 through BX-40 remain unstarted.

### BX-08 implementation checkpoint

BX-08 is executing and uncommitted. Persistent lineage, streamed derivations/blob capture, and explicit governed-prefix recovery pass intermediate tests. Final source-pinned Windows measurements are running; Linux, actual final disk-full/SIGKILL evidence, portable fixture, full gates, archive and publication remain. See the latest DEVLOG for exact diagnostic paths and corrected error history.

### BX-08 local acceptance — publication pending

Bounded lineage history, exact checkpoint recovery, streamed derivations and
explicit governed-prefix recovery passed all local gates. Windows/Linux memory
and abrupt-exit trials, actual disk-full/SIGKILL evidence, portable checkpoint
compatibility, 237/239 Rust tests and 59 Python tests per host passed. The latest
DEVLOG and verification entries retain exact source/archive identities and scope.
BX-08 remains executing until focused main publication and all hosted jobs pass;
BX-09 remains unstarted.

## BX-08 publication verified; BX-09 executing

Date: 2026-09-06 UTC. BX-08 closed at `86061d89a1b3505a8a4a7f77ad679192cafb0b8a`, published on main. [Hosted run 34012687694](https://github.com/USS-Parks/BONSAI-Research-Labs/actions/runs/34012687694) passed all four platform jobs and M1 semantic equivalence. Local and remote main matched, with a clean checkout at publication. The committed-byte check passed for 68 files, 16 captured outputs and the archived recovery evidence.

BX-01 through BX-08 are verified; BX-09 now executes under the existing full-roster STS authority. Extend the existing protocol machine and AdapterConformanceSuite, preserve historical adapter/bundle compatibility, reject unknown required capabilities and breaking epochs, and audit the generic runtime dependency boundary. BX-10 through BX-40 remain unstarted. The BX-08 worktree/storage inventory remains the latest measured inventory; no new worktree or deletion.

## BX-09 — local acceptance passed; main publication pending

Date: 2026-09-06 UTC. BX-09 full local acceptance passed: Windows record `BX-09-WINDOWS-1788671625036769600` (240 Rust / 59 Python tests) and Linux record `BX-09-LINUX-1788671625549464000` (242 Rust / 59 Python tests), with strict lint, schema compatibility and locked generated-binding checks. Windows also passed all documentation and governance checks. Final documentation closeout and exact publication-byte verification follow before commit/main publication and hosted CI.

[Extension contract](../docs/operator/versioned-adapters.md) and `evidence/verification/bx-09/compatibility-matrix.json` record version/required/optional/epoch behavior. Both current and preserved previous bindings passed the same seven-check suite in 24 real process runs across Windows and WSL2 Linux. Reports were byte-identical across hosts. The generic authority source/direct-dependency audit passed; archived governed bundles remained verifiable.

The checked compatibility archive contains 273 payloads and 180 exact source files: 537,499 bytes, SHA-256 `c6eb34c2f45de2f4e7c28079aebfc46679d785fa0daee927c59e8e75baf00320`. The full live report digest is `3a0938ee9bafc3415d8977269dda72af4be0a53bf8a26accce97a443b1281ee2`. No checkpoint restore, scientific-quality, physical-host or hostile-host claim was added.

BX-09 remains executing until its focused commit is on main and all hosted jobs pass. BX-10 through BX-40 remain unstarted. Canonical checkout remains the only active lane; the latest storage/worktree inventory is `evidence/verification/bx-09/worktree-inventory.json`. No worktree or cache deletion.

## BX-09 publication verified; BX-10 executing

Date: 2026-09-06 UTC. BX-09 closed at `4b6ef2f9871f83e24f283dafc0c7f6f0f5fa8b40`, published on main. [Hosted run 34013890557](https://github.com/USS-Parks/BONSAI-Research-Labs/actions/runs/34013890557) passed all four platform jobs and M1 semantic equivalence. Local and remote main matched; the checkout was clean. All 54 committed files, 14 captured outputs, and the archive passed exact publication-byte checks. Native dependency-audit line endings were preserved explicitly.

An automatic approval review initially questioned the push destination/authorization. The established BONSAI remote, prior published parent, continuing source-publication addendum, full-roster authority and exact outgoing payload were verified; review then allowed the original fast-forward push. No credential patterns were found in the 54 files and 274 inspected archive entries. No approval blocker remains.

BX-01 through BX-09 are verified. BX-10 now executes under the existing full-roster STS authority: review reuse/license choices, implement a materially different representation/update procedure at the existing adapter seam, and verify five paired causal-world seeds through the same supervisor, manifest family, resource policies, conformance suite and report path. BX-11 through BX-40 remain unstarted. No new worktree or deletion.

## BX-10 local gates verified — publication pending

Date: 2026-09-06 UTC. BX-10 adds the bounded linear NLMS learner through the extracted shared online adapter, alongside the original exact-observation tabular learner. A fixed action-by-feature matrix and normalized gradient update replace tabular count/return dictionaries and sample-average updates. No external learner dependency was added after the reuse/license review. Generic authority and orchestration contain no learner-specific name/import or branch. Explicit versioned accounting replaces the universal two-touch assumption while preserving historical source/manifest verification.

The final source-frozen matrix `target/bx10-live-1788675720870060478` passed 20 governed runs: seeds 7, 19, 42, 73 and 91, two repeats, both learners, 200 steps each. All used supervisor SHA-256 `867cc9f1ce7ad2dacd02c588282cac348ab0acf3b2f772441ebac8b7c11b58d0`, the same manifest family, resource profile, causal world and report path. Every run passed independent receipt-bound C0/C1 reconstruction. Both actual timeout probes contained and reaped their children. Independent replay checked 4,000 action choices and numeric updates; every repeated learner/seed trace matched exactly. All 20 common-suite certificates passed all seven checks (140 checks). Tabular reported 400 parameter touches/run; linear reported 1,000. Maximum measured step CPU was 1,368,000 ns and maximum agent RSS was 26,869,760 bytes, within common limits; resource groups were separate and cleanup was verified.

Passing machine records: `BX-10-WINDOWS-1788675446991058000` (245 Rust / 63 Python tests), `BX-10-LINUX-1788675338388494900` (247 Rust / 63 Python tests), `BX-10-DEPENDENCIES-1788675369169679000`, `BX-10-LIVE-FINAL-1788675711616435900`, `BX-10-REPLAY-FINAL-1788676542221564000`, and `BX-10-ARCHIVE-1788676569875854900`. Earlier diagnostic failures remain recorded: full-trace ordering timeout, concurrent Windows executable lock, Linux function-length lint, and audit-script line-length lint. The focused total-order fast path preserves all ordering diagnostics and passes a 4,000-event regression. The final matrix was rerun after the source correction.

Retained archive: `evidence/verification/bx-10/paired-learners.zip`, 20,902,457 bytes, SHA-256 `ec8a38d0ff4c01ce438be9ba8367daaf93a38821f41b9a73f777d59ed923db17`, 815 payloads including 207 exact source files. This is virtualized diagnostic conformance and C0/C1 evidence, not scientific-quality, energy, physical-host or hostile-host acceptance.

The worktree inventory records at least 31,944,158,322 bytes of active generated state and 1,463,283,025 bytes in the clean historical M0 worktree (zero unpublished commits). The historical worktree has no active purpose and remains disclosed cleanup debt requiring explicit removal authorization. No worktree/cache deletion or new worktree occurred. BX-10 remains executing until its focused commit/main publication and all hosted jobs pass; BX-11–BX-40 remain unstarted.

## BX-10 publication verified; BX-11 executing

BX-10 closed at `da64e225fa24eaf9b6165ca12d5282c42414b605` on public main. Hosted run [34017607369](https://github.com/USS-Parks/BONSAI-Research-Labs/actions/runs/34017607369) passed all four platform jobs and M1 hosted semantic equivalence. The 72 committed files, 28 captures and final archive passed byte checks. The user explicitly approved public publication of BX-10 and all remaining verified source/evidence commits, including the disclosed workspace paths and hardware metadata; the approved addendum is `docs/governance/addenda/2026-09-06-v05-source-evidence-publication.md`. The original reviewed push then succeeded without a workaround. No publication blocker remains.

BX-01–BX-10 are verified. BX-11 now executes sequentially: optional pinned Gymnasium adapter, direct-versus-adapter seed/action and truncation agreement, regular governed runs with both learners, and core installation without the optional dependency. BX-12–BX-40 remain unstarted. M5b closes only after BX-11 passes its local and hosted gates. No new worktree or deletion.

## BX-11 local gates verified — publication pending

Date: 2026-09-06 UTC. BX-11 adds the optional Gymnasium 1.3.0 FrozenLake-v1 adapter (fixed 4x4 map, deterministic actions, no rendering) through the shared SessionAdapter lifecycle. Only declared observations, rewards and independent termination/truncation flags cross the stream boundary. The dependency is pinned in the optional extra; default core imports and runs remain independent. Source/configuration verification retains historical BX-10 contracts and accepts the new exact environment contract. No new learner or authority branch was introduced.

Four governed runs in `target/bx11-live-1788679369093107551` passed with both learners at horizons 2 and 8, seed 42, 100 steps each, and the original 120-second wall limit. The unchanged supervisor SHA-256 was `da26663ff48ca431cbdbcb5232ea6be5103b3aa98ff2ac82a2398969ab963fd3`. Each receipt passed independent C0/C1 verification. Direct upstream replay matched all 400 actual actions, observations, rewards and end flags. Nine separate direct-versus-framed traces cover three seeds (including a large uint64 seed), goal termination, time-limit truncation, and simultaneous termination/truncation. Both published BX-10 learner bundles also pass the new verifier with their original receipts.

Passing records: `BX-11-LIVE-RETRY-1788679358843095700`, `BX-11-ENVIRONMENT-REPLAY-1788679678405858000`, `BX-11-WINDOWS-1788679696343720000` (245 Rust; 64 Python passed and nine optional tests skipped), `BX-11-LINUX-1788679693161100600` (247 Rust; all 73 Python passed), `BX-11-CORE-INSTALL-1788679851949519600`, `BX-11-OPTIONAL-PROVENANCE-1788679873864763900`, `BX-11-HISTORICAL-1788680098086461200`, and `BX-11-ARCHIVE-1788680161504230900`. Optional dependency versions, upstream URLs, license metadata and lock hash are retained in `evidence/verification/bx-11/dependency-provenance.json`.

The initial diagnostic failed the real wall gate after finalization delay; read-only timing probes measured variable WSL Git overhead (status about 60 seconds, diff about 113 seconds; repeat about 16 and 5 seconds). The unchanged-source retry passed all four cases in approximately 56, 44, 39 and 80 seconds. No limit was relaxed. A core-import probe initially omitted the repository source path; its corrected retry passed. Both failures remain recorded. Evidence-script lint findings were corrected before closeout.

Archive `evidence/verification/bx-11/external-environment.zip`: 1,594,249 bytes, SHA-256 `07cddae7aae3545f6f82631b1b15c62eb7fd770098f0f48bc490346a819b55c6`, 346 payloads, 209 exact source files, nine direct traces and four run bundles. This establishes diagnostic integration and C0/C1 behavior only; it makes no learning-quality, physical-host, energy or hostile-host claim.

The inventory records at least 32,167,446,215 bytes of generated data in the active checkout (some open files unreadable) and 1,463,283,025 bytes in the clean historical M0 worktree, with zero unpublished commits. The historical tree has no active purpose and remains cleanup debt pending explicit removal authorization. No worktree or cache was deleted or created. BX-11 and M5b remain pending focused main publication and all hosted jobs; BX-12–BX-40 remain unstarted. Publication is authorized by the user's explicit 2026-09-06 approval and its recorded addendum.
