# BX-13 execution design

Status: in progress. Canonical scope and acceptance remain BX-13 in the approved v0.5 PSPR. BX-12 closed at f64a38748149ff2df7e68892327f4ae0af0397af with all five hosted jobs passing in run 34047930767.

## Reuse and implementation

The online subproblem and option learner extends the BRDC-1 stages and uses BX-12 OnlineFeatureStage proposals. The existing OrderedAdapter, SessionAdapter, CausalObservation/CausalTransition, PrimitiveAccounting, WorkClass, BudgetAccounts, cgroup authority, immutable run evidence, and verify-run pipeline carry the new execution. The original fixture stages and historical payloads retain their semantics.

The supervisor consumes a generic bonsai.online-accounting/v2 declaration. It admits declared class tariffs before dispatch, transmits an immutable grant, forwards the exact public causal transition, and checks the resulting counters and state limits. It does not know the option policy or BRDC stage classes. Source-specific verification checks the adapter contract separately. The v2 declaration is additive; legacy and v1 peers use their original reward-only feedback and accounting rules. An exact 15-file BX-12 source-set fixture preserves historical verification after adding the new adapter registrations.

The option adapter accepts actions 2–8, observation width 1–8, and an unsigned 64-bit seed. Learner caps are 64 retained public states, four options, and 16 primitive actions per invocation. Empty parameters are updated from single-pass experience. Full-state termination statistics require two observed samples before learned termination. Safety/episode termination remains explicit. Public observations and environmental rewards are the only learning input; the learner receives no finished policy, feature target, private environment configuration, or evaluation labels.

Reservation tariffs per accepted step are acting=actions+4, learning=1, feature_generation=168+4*width, and option_learning=16+4*(8+2*actions+2*width). They are capacity reservations, not measured CPU operations. Actual parameter writes are counted separately. The live profile declares 1 MiB retained learner state, 256 KiB serialized state, at most 1,024 parameter touches per update, and at most 32 KiB update evidence. Existing OS RSS, CPU, wall-time, queue, I/O, and observer limits remain enforced. Memory measurements describe retained Python objects; transient interpreter/allocator peaks are a separate OS measurement.

Primitive-only disables option initiation while retaining discovery, auxiliary learning, and declared tariffs. Reward-oblivious changes only the original-environment-reward term in the option learning cumulant. Both retain the original environmental reward and factual Track A single-pass/no-replay classification. Reward mode and reward-respecting claim eligibility are separate explicit fields. The historical toy comparator's Track-B label is not used to redefine current runtime track semantics. Comparisons use identical conditions and seeds; realized trajectories may differ after a control changes action choice. These diagnostics establish implementation behavior, not scientific efficacy.

The new five-position chain is a BONSAI-owned diagnostic through the existing environment protocol. Only position is exposed. Environment-side reward rules, including state 4 receiving +4 and other arrivals receiving -1, are absent from the agent configuration. Forward/backward actions, horizon truncation, optional goal termination, reset state, reward, and next state are independently checked by the Rust bundle verifier. Continuing episodes allow learned option termination to be observed independently of environment termination.

## Verification sequence

1. Focused learner tests: empty-policy learning, beta support, atomic work/allocation denials, bounded retained state and lineage, detached snapshots, reset, explicit control mechanisms, and variable-duration execution.
2. Real child protocol tests: grant consumption, full public feedback, option initiation/action/termination, exact environmental returns and counters, malformed/absent/insufficient admission, bounded failures and observed process exit.
3. Full locked Windows/Linux repository gates and all existing compatibility fixtures.
4. Source-bound Linux governed runs using the existing unprivileged delegated cgroup supervisor, with measured resource facts, exact source/component identities, retained comparator bundles and learning traces, and observed cleanup. Source and verification records remain frozen during each batch.
5. Independent reconstruction of option updates, lineage, reward/accounting distinction and execution durations from immutable inputs; tampered, missing, contradictory, or forbidden-input evidence must fail.
6. Archive integrity, publication-byte and credential-pattern checks, canonical ledger closeout, parent diff review, fresh independent SHIP acceptance, focused main commit/publication, exact remote SHA and all five hosted jobs before BX-14.

## Ownership and closeout

Parent gpt-6-astra/ultra owns integration and acceptance. The bx13_design delegation requested gpt-5.6-sol/xhigh for bounded learner/tests; bx13_accounting requested gpt-5.6-terra/high for generic Rust/schema contracts. Requested settings are not treated as observed runtime evidence. Every returned runtime/cost result is recorded separately. No new worktree or cache deletion is part of this prompt. Existing physical-host and later long-duration gates remain outstanding.
