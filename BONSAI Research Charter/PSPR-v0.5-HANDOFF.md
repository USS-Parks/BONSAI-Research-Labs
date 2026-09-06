# PSPR v0.5 Project Handoff

Date: 2026-09-05 (America/Los_Angeles)
Status: **APPROVED — STS RUNNING; BX-01–BX-12 VERIFIED; BX-13 IN PROGRESS**

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

## BX-11 publication verified; BX-12 executing

BX-11 closed at `e18dabdbcfa8135a8a078a629fa7aee375d149a5` on public main. Hosted run [34019743668](https://github.com/USS-Parks/BONSAI-Research-Labs/actions/runs/34019743668) passed Windows x64, Linux x64, macOS ARM64, macOS x64 and M1 semantic equivalence. The optional Gymnasium integration passed on every platform. All 60 committed files, 22 captures and the 1,594,249-byte archive passed exact publication-byte checks; publication review found no credential patterns in the 60 files and 347 archive entries. Local main and origin/main matched and the checkout was clean. M5b is complete.

BX-01–BX-11 are verified. BX-12 now executes under the continuing full-roster STS and public source/evidence authorization: extend the existing FeatureStage with bounded online candidate construction, deterministic proposals, explicit disabled controls, lineage/resource evidence and pre-mutation work/allocation denials. BX-13–BX-40 remain unstarted. No new worktree or deletion.

## BX-12 restart checkpoint — standby at user request

The user requested: "Get to a logical stopping point and standby so I can restart the app with the new astra-advisor plug-in." Execution is paused at this checkpoint. Resume only when the user returns; do not start another prompt or background continuation during standby. Full-roster STS and the 2026-09-06 public source/evidence publication authorization remain valid.

BX-01–BX-11 are complete and verified. Local main and origin/main were verified at `e18dabdbcfa8135a8a078a629fa7aee375d149a5`; BX-11 hosted run `34019743668` passed all five jobs. BX-12 remains IN PROGRESS (`[~]`), uncommitted, with no staged changes or unpublished commits. BX-13–BX-40 remain unstarted. All launched commands and the read-only BX-12 review agent have finished.

### Preserved BX-12 implementation

New untracked source:
- `python/bonsai-reference/src/bonsai_reference/feature_discovery.py`
- `python/bonsai-reference/src/bonsai_reference/feature_admission.py`
- `python/bonsai-reference/tests/test_feature_discovery.py`
- `crates/bonsai-governor/examples/feature_admission.rs`
- `crates/bonsai-contracts/examples/feature_lineage.rs`
- `docs/research/online-feature-discovery.md`
- `evidence/verification/bx-12/generate.py` and `live.py`

The online FeatureStage subclass constructs bounded equality features from observed experience, records birth/revision lineage and measured representation bytes, and commits prospective state only after actual external work and allocation admissions. The Rust helper reuses WorkAllocator and StorageBroker. The second Rust helper validates existing artifact wire lineage. Disabled controls, deterministic replay, and denial at a populated revision boundary are covered. Original tiny FeatureStage fixtures remain intact.

Review fixes preserve read-only configuration and detached snapshots, reject inherited mutation bypasses, include the root object in retained allocation measurement, reap governor subprocesses while preserving transport errors, and reuse existing storage reservations for zero or shrinking growth. The fixed work tariff is a reservation policy, not measured CPU work; retained Python object size is not OS RSS.

All four canonical documents and `evidence/verification/records.jsonl` are modified; BX-12 source, evidence captures and the storage inventory remain untracked. Preserve this tree and its evidence; continue the existing implementation.

### Checks completed before standby

- Focused Windows regression record `BX-12-RESTART-CHECK-1788710319551226700`: **21 passed**, 1.42 seconds, including actual governor work/memory/storage denial with unchanged populated learner state.
- `uv run --frozen ruff check .`: passed. `uv run --frozen pyright`: zero errors (run before the final test parameterization expansion).
- Both Rust examples built; `cargo clippy --locked --offline -p bonsai-governor -p bonsai-contracts --examples -- -D warnings`: passed.
- Diagnostic record `BX-12-LIVE-DIAGNOSTIC-1788710123737389900`, batch `target/bx12-live-1788710125347254100`: nine runs, seeds 7/19/42, repeated enabled and disabled controls, 128 steps each. Trajectories matched controls; enabled state/lineage repeated exactly; existing Rust lineage validation passed.
- The diagnostic preceded the latest source-binding and actual governor PID/return-code recording changes in `live.py`. Those changes are saved but have not been exercised. The earlier batch is diagnostic evidence, not final publication proof.

No full BX-12 platform gate, final archive, commit, publication, or hosted CI has completed. No BX-12 completion claim is made.

### Exact next work after restart

1. Read this checkpoint and the BX-12 roster gate, inspect the preserved diff, and continue BX-12. Add CI builds for the Rust feature-admission helper before Python tests so helper-dependent coverage cannot silently skip; include lineage helper/build coverage as needed.
2. Complete Windows/Linux gates using existing environments. For Linux, set `BONSAI_FEATURE_GOVERNOR` to `target/x86_64-unknown-linux-gnu/debug/examples/feature_admission` and `BONSAI_FEATURE_EXAMPLES` to that examples directory. Reuse `target/linux-venv`; do not duplicate environments.
3. After source changes finish, rerun the final nine-run source-bound matrix. Preserve the prior diagnostic. Do not mutate source, documents or verification records during the source-bound run. Confirm before/after source hashes and actual governor PID/exit/reaping evidence.
4. Create and verify the final archive of exact source, proposal traces, lineage/resource records, controls and denial evidence. Apply necessary LF attributes and verify capture/archive bytes.
5. Close the prompt ledgers, commit and publish to main under existing authorization, verify the remote SHA and all five hosted jobs, then proceed to BX-13. Do not repeat an equivalent publication approval request.

Windows command execution previously required explicit `C:\Windows\System32\cmd.exe` with login disabled; bundled pwsh failed. The temporary automatic approval-review capacity rejection was resolved through a normal-path retry after refreshed usage evidence. No reset credit was redeemed and no current blocker remains from that rejection.

### Retained worktrees and storage

Inventory: `evidence/verification/bx-12/restart-storage-inventory.json`. The active canonical checkout owns BX-12, is dirty as described above, and has zero unpublished commits. Its target contains at least 32,209,834,710 bytes (44 unreadable files), plus a 65,616,320-byte venv: at least 32,275,451,030 bytes total.

Historical `C:/tmp/bonsai-m0-sts`, branch `codex/m0-governed-foundation`, is clean at `eaa0e52ec5a6dc78ab1a360f2a11c2201c7a5e9d`, with zero unpublished commits and 1,463,283,025 bytes of generated data. It has no active purpose and remains cleanup debt pending explicit removal authorization. No worktree, cache or artifact was deleted; no new worktree was created.

## 2026-09-06 — BX-12 resumed with Astra Advisor

The user returned after the app restart and instructed continued work on the same trajectory using astra-advisor:orchestration. This supersedes the standby checkpoint without changing the approved BX-01–BX-40 scope or publication authority. BX-12 resumes from its preserved uncommitted tree; BX-13–BX-40 remain unstarted. Runtime turn metadata confirms parent gpt-6-astra / ultra. The parent owns acceptance; a bounded gpt-5.6-sol / high native delegation owns only CI integration. Full platform verification, source-bound diagnostic evidence, independent reconstruction, archive integrity and a fresh read-only ship review remain required before publication.

## BX-12 execution baseline reconciled — 2026-09-06

Live remote main advanced by one reviewed capture-attribute/governance commit, 798d8bf0a06d8a7aa2b4e07d32db3b6898cedd5e (PR #5). After the initial Windows and Linux gates passed, local main fast-forwarded to that commit. All 37 preserved BX-12 source, document, harness and captured-output files passed exact SHA-256 preservation checks; the BX-12 attribute appendix was retained and reapplied. No implementation was discarded, no stash/worktree created, and no history rewritten. Final evidence and acceptance use this combined baseline.

## 2026-09-06 — BX-12 local verification and evidence ready for review

BX-12 remains in progress pending fresh acceptance review, focused main publication and hosted CI. The preserved implementation extends FeatureStage with bounded online equality-feature proposals, immutable birth/revision identities, observed exposure summaries and explicit work/storage admissions. No representation, semantic label or utility target is supplied to the online API. Original legality fixtures remain unchanged.

Files: the feature_discovery and feature_admission Python modules; test_feature_discovery.py; Rust feature_admission and feature_lineage examples; .github/workflows/ci.yml; .gitattributes; docs/research/online-feature-discovery.md; the BX-12 verification harness, archive and captures; and these four canonical execution documents. CI builds the actual helpers before Python tests, generates pinned protobuf bindings, runs the live diagnostic and uploads each platform's evidence.

Parent verification: `BX-12-WINDOWS-RECONCILED-1788713832980061400` passed 245 Rust tests and 85 Python tests with nine optional Gymnasium skips; `BX-12-LINUX-RECONCILED-1788713838523383900` passed 247 Rust tests and all 94 Python tests, including optional Gymnasium coverage. Formatting, workspace Clippy, schemas, Python lint/types, protocol generation and applicable governance checks passed. The 21 focused BX-12 cases include work, memory and storage denial at empty and populated revision boundaries, unchanged authoritative state, disabled controls, deterministic proposals, bounded growth, public-mutation rejection and actual child cleanup. `BX-12-EVIDENCE-LINT-1788713984124473800` passed after the final harness changes.

Final live records `BX-12-LIVE-WINDOWS-1788714006795931800` and `BX-12-LIVE-LINUX-1788714033939785500` passed. Batches `target/bx12-live-1788714007535274100` and `target/bx12-live-1788714036759617101` each contain nine runs of 128 steps: seeds 7, 19 and 42, two enabled repeats and one disabled control. All 18 actual governor processes exited zero and were reaped. Independent reconstruction verified all 2,304 steps, exact feature state/proposals, wire lineage/provenance and work/storage records. Changed reward, missing step, false work, altered representation, supplied utility, corrupted state, missing lineage and inconsistent track metadata were rejected on both hosts. All nine trajectory/state/lineage hashes matched across Windows and Linux; each host retained its own measured allocation and process facts.

Archive `evidence/verification/bx-12/online-features.zip`: 1,130,567 bytes; SHA-256 `4d3012e4e11a77e0b2115fdd1fa6ccc3f214ea56171196a316e3a32314c8bbee`; 367 payloads, including 245 exact source files, 18 diagnostic runs and generated audit bindings. `BX-12-ARCHIVE-1788714125663811000` passed complete payload length/hash verification. The archive source manifests agree across both hosts and were checked against current source bytes. Source revision is `798d8bf0a06d8a7aa2b4e07d32db3b6898cedd5e` with the retained BX-12 source snapshot; the focused commit SHA will be recorded after publication using the established subsequent-entry convention.

The initial Linux launch with an invalid saved username failed before running tests and remains recorded as `BX-12-LINUX-1788713569872597500`; retry with Ubuntu's configured default user passed. The earlier diagnostic and restart captures remain historical evidence. Windows is local/unattested; Linux is explicitly WSL2 (`6.6.87.2-microsoft-standard-WSL2`), not physical acceptance. This prompt demonstrates experience-driven candidate construction and governed mutation, not downstream utility, C2–C5 success, OS RSS limits or physical-host qualification.

Current storage inventory records at least 32,494,441,786 bytes in the canonical target/venv (48 unreadable files). The clean historical M0 worktree retains 1,463,283,025 bytes of generated data and zero unpublished commits; its contents are ancestors of verified remote main. It has no active purpose and remains cleanup debt pending explicit removal authorization. No worktree or cache was created or deleted.

## 2026-09-06 — BX-12 independent acceptance review

A fresh read-only review by `/root/bx12_acceptance` returned **ship**, with no findings. Observed runtime: gpt-5.6-sol / xhigh, from native turn metadata; parent gpt-6-astra / ultra retained acceptance authority. The reviewer inspected the real change set and independently checked both platform results, all 18 governor exits, 2,304 reconstructed steps, eight negative evidence cases per host, nine matching cross-host semantic cases, all 245 current source hashes, all 367 archive payloads, and all 30 capture references then staged. Read-only conduct was procedural, not a separate enforced sandbox.

The reviewer correctly left hosted CI pending and identified the two subsequent publication-scan records/four captures for parent closeout. The corrected final scan `BX-12-PUBLICATION-SCAN-1788714373380706700` found no credential-pattern matches in 56 staged files and 368 archive entries. Public source/evidence publication remains authorized by the existing 2026-09-06 addendum; no release, package upload or physical-acceptance claim is implied.

API-EQUIVALENT COST RECEIPT: unavailable for this BX-12 resumed segment. Native parent, CI delegate and reviewer tooling did not expose complete per-call input, cached-input and output token usage or pricing eligibility. The historical pricing reference is the 2026-09-04 snapshot (Sol pricing promotional); no prices were applied and no cost, savings or subscription-charge claim is made.

## 2026-09-06 — BX-12 published and verified; BX-13 started

BX-12 closed at `f64a38748149ff2df7e68892327f4ae0af0397af` on public main. Hosted run [34047930767](https://github.com/USS-Parks/BONSAI-Research-Labs/actions/runs/34047930767) passed Windows x64, Linux x64, macOS ARM64, macOS x64 and M1 semantic equivalence. Each platform ran the live feature diagnostic and uploaded its evidence. The final index and committed-HEAD checks verified all 62 files, 36 captured outputs and the archived bundle with zero byte mismatches; the final credential-pattern scan found no matches in 62 files and 368 archive entries. The commit hook and publication hook passed. Live remote main exactly matched the full commit SHA and the checkout was clean after publication.

BX-01–BX-12 are verified. BX-13 now executes under the continuing full-roster STS and public source/evidence authorization. It will extend the existing subproblem and option stages with experience-learned policies and termination, route actual execution through the ordered run protocol, and retain separate environmental reward, work admissions, resource measurements, learning traces and primitive-only/reward-oblivious controls. BX-14–BX-40 remain unstarted. Parent Astra retains acceptance authority; fresh independent SHIP review remains required before publication. No new worktree, cache deletion, physical-acceptance claim or scientific efficacy claim.


## 2026-09-06 — BX-13 parent verification and evidence freeze

BX-13 remains in progress pending the final governed matrix, independent reconstruction, archive integrity, fresh SHIP review, publication and hosted CI. Parent review corrected v2 accounting to treat maximum parameter touches as a ceiling while retaining exact legacy counts; six contract regressions pass. The 32 focused learner/process cases pass, including populated-state work/allocation denial, variable learned duration, episode/duration termination and contradictory causal feedback refusal with actual child cleanup. The populated storage test covers four options, 64 public states, eight actions and all eight supported observation coordinates.

Windows gate `BX-13-WINDOWS-1788717025064418400` passed 248 Rust tests, 117 Python tests with nine optional Gymnasium skips, formatting, lint, strict types, schemas, protocol and governance. After expanding the maximum-width test, `BX-13-WINDOWS-PYTHON-FINAL-1788717335940284000` passed the same 117 tests and nine optional skips. Linux gate `BX-13-LINUX-LOOP-EXTRACTION-1788717200259687800` passed 250 Rust tests and all 126 Python tests including Gymnasium. The initial Linux gate found a 106-line loop exceeding the existing Clippy limit; extracting unchanged storage-policy construction fixed it without a suppression or policy change. The retained initial Windows temp-directory, manifest base-seed and concurrently changed tracked-patch failures remain visible in the verification records.

Provisional governed run `BX-13-LIVE-PROVISIONAL-STABLE-PATCH-1788716840074923000` completed 256 steps and 5,444 events. The ordinary independent verifier accepted its receipt; separate reconstruction reproduced all steps and rejected ten altered-evidence cases. Observed step CPU peaked at 6,257,000 ns under the unchanged 10,000,000 ns limit; completed learned-option durations included 1, 2, 3, 4, 6 and 8 actions. This provisional evidence is superseded for acceptance by the pending full matrix. Recorded owned-object bytes are checked for bounds and consistency; canonical state bytes/hashes and learning updates are independently reconstructed.

`BX-13-HISTORICAL-1788717594114662600` verified all four published BX-11 bundles after exact comparison with their archived bytes. `BX-13-CI-PROTOCOL-LOCAL-1788717606337505000` passed all 11 actual-process protocol cases and retained JUnit, outputs and source/host identity; it makes no OS-controller or physical-host claim. CI preserves all five required jobs and runs/uploads this evidence on every platform.

Astra delegations returned: bx13_design and bx13_replay requested Sol/xhigh, bx13_accounting requested Terra/high, with actual runtime unavailable; bx13_ci ran on observed Sol/high (native metadata, thread 01a077a0-0b68-79e2-a958-85195525476e). Parent gpt-6-astra/ultra owns acceptance. API-EQUIVALENT COST RECEIPT: unavailable for this segment because native per-call token usage was not exposed; no cost, savings or subscription-charge estimate is made.

The final matrix retains seeds 7, 19 and 42, two option-enabled respecting repeats, one primitive-only and one reward-oblivious control per seed, plus an environment-termination case: 13 runs of 256 steps, horizon 32. All CPU, action, memory, I/O and wall limits remain active. Source, tracked patch and records are frozen during the matrix. WSL remains virtualized development evidence; scientific efficacy and physical acceptance are not claimed. No new worktree or deletion.


## 2026-09-06 — BX-13 complete local evidence and parent acceptance

Final governed record `BX-13-LIVE-FINAL-1788717712791955600` passed all 13 runs of 256 steps in `target/bx13-live-1788717713189903230`; all 26 agent/environment children exited and their cgroups were empty. The harness verified unchanged source bytes, tracked patch and operator binary across the batch. The frozen source identity contains 258 files; operator SHA-256 is `7a8b678cdb69e052f1a4b0dce778ca46d35ea02259e4d86cecb2e2e42d1c0068`. The declared custom diagnostic profile retained 120-second per-run wall, 50 ms action, 10 ms step CPU, 1 GiB RSS, 64 MiB agent storage and 512 MiB observer-output limits. No Smoke/Conformance/Acceptance/Long profile completion is claimed.

Independent record `BX-13-REPLAY-FINAL-1788718292448615600` reproduced all 3,328 steps from immutable public transitions and zero initial parameters, including feature revisions, subproblem/option identities, Q/visit/beta deltas, full state hashes, canonical sizes, execution traces, separate reward returns, all four class tariffs and parameter touches. Ten altered-evidence cases were rejected. All three repeat pairs matched trajectory, state and execution hashes. Completed option durations were 1, 2, 3, 4, 5, 6, 8, 10, 12, 13, 14, 15 and 16 actions; learned beta, duration cap, environment truncation and environment termination were all observed. Of 915 initiations, 914 completed. One invocation in the environment-termination case remained active at global run stop and is explicitly recorded as censored; no option termination was invented.

Measured maxima across the matrix were 9,330,000 ns step CPU, 36,139,008 bytes process RSS, 13,523 retained learner-object bytes and 4,297 canonical state bytes. Owned-object bytes remain host measurements checked for admission/audit consistency and caps; they are not rederived from JSON or presented as RSS. Reconstruction retains per-step learning curves and reports matched control conditions without claiming identical realized trajectories or scientific efficacy. All runs remain factual single-pass Track A with zero replay; reward-oblivious eligibility is explicit. WSL is not physical acceptance.

Archive `evidence/verification/bx-13/online-options.zip`: 16829864 bytes; SHA-256 `030b972470cf36f7ea252180acce8226da28df6ef8f9b8af6bfb8cc73e1e631c`; 630 payloads, including all 258 source files, 13 governed bundles, complete independent reconstruction, generated audit bindings, all four historical verification results and the locally verified CI protocol outputs/JUnit. `BX-13-ARCHIVE-1788718311716389700` passed every payload length/hash and archive integrity check. Parent inspected the actual learner, adapters, generic supervisor/accounting, independent verifier and reconstruction changes. Fresh independent SHIP review, final publication-byte/credential scan, main publication and five hosted jobs remain pending; BX-13 stays [~] until those close.


## 2026-09-06 — BX-13 retained worktree and storage closeout

Inventory `evidence/verification/bx-13-worktree-inventory.json` records the canonical main checkout at BX-12 commit `f64a38748149ff2df7e68892327f4ae0af0397af` with the pending BX-13 patch and zero unpublished commits. Its generated target and virtual environment occupy at least 32,894,213,135 bytes (30.64 GiB); 58 unreadable temporary/cache directories make this a lower bound. These retained builds and verification batches support the active sequential lane. The historical `C:/tmp/bonsai-m0-sts` worktree remains clean at `eaa0e52ec5a6dc78ab1a360f2a11c2201c7a5e9d`, branch `codex/m0-governed-foundation`, with zero unpublished commits relative to origin/main and 1,463,283,025 bytes (1.36 GiB) generated data. It has no active task purpose; retirement remains cleanup debt pending explicit deletion authority. Parent cross-checked both worktrees and remote identity. No worktree was created or removed and no cache was deleted. C: had 288,591,392,768 bytes free at the inventory snapshot.

Astra result: bx13_storage completed the bounded read-only inventory, requested Luna/medium; actual model/effort and per-call token usage were not exposed. Parent accepted the measured inventory with its stated unreadable-path limitation. API-EQUIVALENT COST RECEIPT: unavailable; no cost or savings estimate is inferred.


## 2026-09-06 — BX-13 independent SHIP and publication readiness

Fresh independent reviewer `/root/bx13_acceptance` returned SHIP with no blocking implementation, accounting, evidence or regression finding. Review covered the actual staged learner, adapters, v2 contracts, supervisor, independent verifier/reconstruction, controls, tests, CI and evidence. Its fresh replay passed all 13 runs and 3,328 steps, rejected ten mutations and retained one censored invocation; its independent index check passed 81 files and 42 captures with zero mismatches. Parent accepted the verdict after inspecting the actual changes and completing the full Windows/Linux, governed-run, reconstruction, historical and archive gates. Final governance record `BX-13-CLOSEOUT-FINAL-1788718714032382300` passed. Parent also confirmed all 258 runtime source hashes still match the frozen source identity; the 81-file/631-archive-entry credential-pattern scan had no findings.

Reviewer routing requested Sol/xhigh; native model/effort and per-call usage were unavailable. It made no source, index, commit or worktree mutation. API-EQUIVALENT COST RECEIPT: unavailable across parent, workers and final reviewer because billing-grade per-call usage is not exposed; receipt coverage is partial and no cost, savings or subscription-charge estimate is inferred.

BX-13 is ready for the focused authorized main commit and public push. It remains [~] until the exact remote commit and all five hosted jobs are verified. The measured 9.33 ms step-CPU maximum remains below the unchanged 10 ms bound with limited margin. Hosted adapter protocol tests establish cross-platform process behavior; they do not establish OS-controller or physical-host acceptance. WSL and local evidence remain diagnostic and no scientific efficacy claim is made.
