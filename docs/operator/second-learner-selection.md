# BX-10 second learner selection

The selected baseline is a small linear contextual reward predictor with a
normalized online squared-error update. It differs from PrimitiveTabularControl
in both representation and update: a fixed action-by-feature weight matrix replaces
the exact-observation count/return dictionaries, and a gradient step replaces the
sample-average update.

## Reuse and license review

| Candidate | Evidence | Decision |
| --- | --- | --- |
| Existing PrimitiveAdapter, OrderedAdapter, transport, work query and reports | Project source at BX-09 main; MIT OR Apache-2.0 | Extract the controller seam and reuse all protocol handling |
| Existing comparator module | Comparator declarations, not a second executable learner | Retain declarations; they do not meet this prompt's distinct-learner gate |
| Vowpal Wabbit | [Official contextual-bandit tutorial](https://vowpalwabbit.org/docs/vowpal_wabbit/python/latest/tutorials/python_Contextual_bandits_and_Vowpal_Wabbit.html), [BSD three-clause license](https://github.com/VowpalWabbit/vowpal_wabbit/blob/master/LICENSE), reviewed 2026-09-06 | Credible reusable baseline with a permissive license, but its native dependency/build surface and richer accounting exceed this small conformance cut; no code or dependency imported |
| Small project-local linear learner | Original implementation of a standard online regression idea | Selected; no third-party code copied; project dual license applies |

The scientific rationale is online regression from chosen-action feedback, a
standard contextual-bandit construction discussed by Bietti, Agarwal and Langford,
[A Contextual Bandit Bake-off, JMLR 22 (2021)](https://jmlr.org/papers/v22/18-863.html).
This implementation is a deliberately small conformance baseline, not a replication
of their benchmark or a claim of their performance guarantees.

## Settled implementation

For an observation of fixed width d, use a bias and bounded scalar coordinates:
phi = (1, x_1/(1+x_1), ..., x_d/(1+x_d)). Inputs are unsigned protocol coordinates;
there are no environment state labels, reward-defined feature targets, replay,
or learned feature construction. Each action owns d+1 weights initialized to zero.
Choose each action once initially, then greedily maximize its dot product, with
lowest-index ties. One chosen-action reward produces:

```text
prediction = dot(weights[action], phi)
weights[action] += 0.25 * (reward - prediction) * phi / dot(phi, phi)
```

The pending feature vector/action is consumed exactly once. Episode reset preserves
weights and requires no outstanding feedback. Dimensions and hyperparameters are
fixed for a run. Greedy exploration can fail; the gate demonstrates flexibility
and actual updates, not superior learning.

Reuse the existing coarse work units: one action evaluation is one acting work
item and one online update is one learning work item. These units are not FLOPs.
Parameter touches report actual parameter cells updated (two count/return cells
for the tabular learner, d+1 weights for the linear learner). Actual CPU and RSS
remain separately measured and governed under identical paired-run policies.

The generic runner must consume an explicit versioned accounting declaration
instead of assuming two parameter touches for every possible learner. Historical
manifests retain their established v1 tabular accounting interpretation. The
independent scientific verifier must bind each supported implementation's source
identity and accounting declaration; generic runtime/governor/platform authority
must not branch on learner identity.

## Acceptance still required

Five paired seeds on the same causal world, same supervisor binary, manifest family,
resource policies, conformance suite and report path. Retain actual learning/update
differences, separate resource records, strict source identity, archived verification,
negative accounting cases, full local gates and hosted publication. No implementation
or execution result is implied by this selection note.

## Local acceptance evidence — 2026-09-06

The final five-seed paired matrix passed for both learners and both repeats: 20 runs, 4,000 steps, independently replayed numeric updates, and 140 passing common conformance checks. Retained evidence: `evidence/verification/bx-10/paired-learners.zip`, SHA-256 `ec8a38d0ff4c01ce438be9ba8367daaf93a38821f41b9a73f777d59ed923db17`. Full Windows and Linux regressions passed; development and verification logs retain exact records and earlier failed attempts. Main publication and hosted CI remain the prompt closeout gate.
