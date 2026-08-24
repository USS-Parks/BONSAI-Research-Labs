# BONSAI Reference Discovery Cycle v1

BRDC-1 is a STOMP/OaK-style reference cycle built only from public tabular and linear ingredients. It is not an Oak Lab implementation, not a reproduction of unpublished algorithms, and not a deep or open-ended success claim.

The cycle exists so BONSAI can measure feature, subproblem, option, model, planning, credit, scheduler, and curation telemetry under Track A constraints: batch size one, no agent-side replay, and an external governor that remains authoritative.

## Attribution boundary

| May be used | Must not be claimed |
|---|---|
| Public options (Sutton, Precup, Singh), reward-respecting subtasks (Sutton et al., 2022), tabular/linear models, Dyna-style planning, and published GVFs | Unpublished Oak Lab algorithms or “the OaK solution” |
| Explicit BONSAI design choices labeled as such | Deep, nonlinear, or open-ended competence |
| Experimental hypotheses that later prompts may confirm or refute | Instrument completion or C0–C5 pass |

Feature discovery remains an unresolved public seam. BRDC-1 therefore uses only declared public-ingredient feature candidates and records their lineage; it does not treat candidate creation as utility.

## Cycle stages

1. **Feature candidates.** Emit or revise integer feature detectors over the public observation. Birth, revision, and retirement are logged. Creation is cost, not benefit.
2. **Reward-respecting subproblems.** For a selected feature, pose a feature-attainment task whose success uses original reward plus a published stopping bonus. Reward-oblivious tasks are labeled and never reclassified as reward-respecting.
3. **Options.** A solved subproblem yields a policy plus a stopping rule. Unused and harmful options remain first-class records.
4. **Option models.** Learn one-step and option-horizon consequences online, batch one, without replay. Errors compare only under lineage-aligned targets.
5. **Planning.** Use primitive and option models under an external backup budget. A backup is consequential only under D-19.
6. **Backward utility credit.** Return slower consumer evidence to upstream artifacts. Exact leave-one-out or matched ablation may later support C3; proxy credit cannot.
7. **Scheduler events.** Emit named stage events for observer accounting. The internal scheduler does not self-certify compliance.
8. **Curation.** Retain, deprioritize, replace, or remove artifacts from a declared estimator. The governor, not the estimator, enforces budgets.

## Traceability table

Every mechanism distinguishes a public basis, a BONSAI design choice, and an experimental hypothesis. The hypothesis column is not a result.

| Mechanism | Public basis | BONSAI design choice | Experimental hypothesis |
|---|---|---|---|
| Feature candidates | Public observation features and the unresolved feature-generation seam (OaK register; Javed and Sutton big-world challenge) | Integer detectors with immutable lineage, cost, and consumers; human-semantic names are not utility | Diagnostic worlds will show whether candidate revision and retirement can be reconstructed without labels |
| Reward-respecting subproblems | Sutton et al., *Reward-Respecting Subtasks* (arXiv:2202.03466): original reward plus feature stopping bonus | Track A poses only reward-respecting attainment tasks; oblivious controls stay on comparator tracks | Alignment `success iff original reward > 0` remains distinguishable from oblivious success |
| Options | Sutton, Precup, and Singh options: policy plus termination | Options are born only from selected subproblems; creation is not benefit | Reliable, redundant, harmful, and unused classes will reconstruct from executions and marginal gain |
| Option models | Public option-model / GVF prediction of outcome, reward, and duration | Online sample-average tables, batch one, no replay; unaligned targets refuse numeric comparison | A known tabular fixture will meet a declared tolerance in one pass or report failure honestly |
| Planning | Public Dyna-style use of models; temporally extended jumps as a published OaK-style idea | External backup budget; D-19 consequentiality; value change is not sufficient | Paired omit-one traces will separate consequential and value-only backups |
| Backward utility credit | Public statement that each forward dependency has a slower backward utility flow | D-20 hierarchy with tier labels; consumer credit is not C3-eligible alone | Forward construction will precede slower credit on exact diagnostic traces |
| Scheduler events | Public eight-process concurrency as a measurement obligation, not an algorithm | Observer-owned named events; internal self-report cannot enforce compliance | Work and consequence counts will reconcile with governor-visible budgets |
| Curation | Public need to retain useful structure under limited memory | Declared estimator proposes dispositions; governor remains authoritative | Useful, redundant, and stale fixtures will receive expected dispositions without mutating lineage |

## Non-claims

BRDC-1 does not implement unpublished Oak Lab code, does not complete feature discovery, and does not assert deep or open-ended success. Later BE prompts implement this specification; later BV prompts adjudicate C0–C3 against evidence. Failure of BRDC-1 on a diagnostic world is a valid instrument result.
