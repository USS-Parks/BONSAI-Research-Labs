# Online option learning contract

BX-13 adds a bounded reference learner at
`python/bonsai-reference/src/bonsai_reference/option_learning.py`. It extends
the existing BRDC `SubproblemStage` and `OptionStage` seams while leaving the
historical fixture implementation unchanged. The online stage never calls the
legacy `OptionStage.solve` method and receives no policy, termination table,
goal label, target action, or completed parameter set.

This is a mechanism and protocol result. It does not establish a scientific
efficacy claim.

## Public control API

```python
OnlineOptionControl(
    actions: int,
    width: int,
    seed: int,
    *,
    options_enabled: bool = True,
    reward_mode: Literal["respecting", "oblivious"] = "respecting",
    max_states: int = 64,
    max_options: int = 4,
    max_duration: int = 16,
)
```

Version 2 accepts 2--8 actions, observation width 1--8, and an unsigned
64-bit seed. The three capacity arguments are present in the configuration and
state snapshot, but this version accepts only 64 states per option, four
options, and duration 16. The properties are read-only.

The control methods are:

- `reset_episode()`, which preserves learned parameters and rejects a reset
  with an unconsumed action or active option;
- `act(observation, admission) -> int`, which returns one primitive action;
- `observe(reward, next_observation, terminated, truncated,
  source_event_id, admission) -> dict`, which consumes the one pending causal
  transition and returns the exact decoded audit carried in
  `parameter_update`;
- `online_snapshot()` and `allocated_bytes()`, which expose bounded state for
  allocation checks and independent reconstruction; and
- `accounting` and `parameter_update`, which fit the existing
  `PrimitiveAccounting` response fields.

The update index is zero-based inside the control. An `act`/`observe` pair owns
internal index `N`; the audit reports one-based `update=N+1`. The feature stage
receives `N`. No pair may begin at or above `2**32`.

The admission object provides
`work(work_class, request_id, amount)` and
`allocation(request_id, allocated_bytes, serialized_bytes)`. A denial raises a
stable `CycleError`. The learner remains byte-identical to its state before the
denied call. The external authority may still account reservations admitted
earlier in that attempted pipeline.

## Learning rules

The BX-12 online feature stage receives the next public observation and raw
environment reward. Its fixed configuration is 32 exposure statistics and
four equality features. A feature birth deterministically creates one
subproblem and one zero-initialized option, subject to the four-option cap.
Every feature revision is linked exactly in its update audit. Persistent
subproblem state retains only the birth revision, latest revision, and revision
count, so lineage cannot become a lifetime-sized list. A new option is not
updated from the same transition that created it.

For each older option, the controller performs one off-policy tabular update
from the observed primitive action. With stopping bonus `b=1`, the cumulant is

```text
respecting: c = environmental_reward + b * attained(next_state)
oblivious:  c =                        b * attained(next_state)
```

If the next state has not attained the feature and the environment has not
terminated or truncated, the target also includes the maximum learned value
at that next state. The learning rate and discount are exactly one. Q values,
reward sums, and returns use checked integer arithmetic; Q values must remain
inside signed 32-bit magnitude. Initial Q values are zero. Action selection
explores each unseen state/action cell, applies deterministic periodic
continuing exploration, and otherwise selects the highest learned value with
a stable action-index tie break.

Termination is learned for the complete public observation tuple. Each option
retains `(positive_count, total_count)` for at most 64 states. Learned beta is
true only when `total_count >= 2` and
`positive_count * 2 >= total_count`. The attained feature is evaluated when
updating these counts, but an unseen state that activates the feature cannot
terminate an option on its first sample. This prevents an activation-only
predicate from masquerading as learned termination.

An executable option must have at least one observed attainment, at least one
beta update, and at least as many Q updates as primitive actions. Initiation,
each emitted primitive action, continuation, and termination carry a stable
invocation ID. Termination reasons distinguish learned beta, environment
termination, environment truncation, and the duration cap. The raw
environment return and the subproblem return remain separate throughout.

The controller retains parameters and sufficient statistics only. It retains
zero replay items and accepts no replay input.

## Work and storage accounting

Every accepted environment step reserves the following deterministic capacity
tariffs in this order across the action and feedback calls:

| Work class | Reservation per step |
| --- | ---: |
| `acting` | `actions + 4` |
| `learning` | `1` |
| `feature_generation` | `168 + 4 * width` |
| `option_learning` | `16 + 4 * (8 + 2 * actions + 2 * width)` |

These are declared capacity reservations. They are not CPU-time, instruction,
or wall-clock measurements. Option construction and later maintenance use the
same `option_learning` tariff and are identified separately in the audit.

`WorkAccounting.environment_steps` and `updates` equal the number of accepted
causal feedbacks. `work_items` is the cumulative sum of all four admitted
classes. `parameter_touches` counts committed learned writes: two primitive
sufficient-statistic cells when the primitive state is admitted; two exposure
cells per admitted feature statistic; the proposal's declared feature
parameter count; and four option cells (Q, action visit count, beta positive
count, and beta total count) for each option whose bounded state could accept
the current and next observations. It is separate from reserved work. Replay
is always zero.

Primitive statistics and each option's state tables stop admitting new states
at 64 entries. The persistent learner state must remain within 1 MiB retained
and 256 KiB canonical serialized. One update may touch at most 1,024 parameters. The exact
canonical update must remain within 32 KiB. Protocol response caches, including
the last audit and the admitted act reports, are excluded from the
persistent-state hash and learner allocation so the report is not
self-referential. Those caches are independently bounded: the audit by 32 KiB
and admitted reports by fixed selected fields. `allocated_bytes()` walks the
actual unique Python-owned learner object graph, including nested stage
objects, slot dataclasses, tables, lists, tuples, and sets; it does not estimate
allocation by materializing a JSON snapshot. Process RSS remains an outer
runtime measurement rather than a learner-state measurement.

An accepted update has exactly these top-level fields:

```text
schema, update, action, reward, work_by_class, parameter_touches,
state_before, state_after, allocated_bytes, serialized_bytes, details
```

`schema` is `bonsai.online-update/v2`. The `details` object contains the full
causal transition and source event ID, feature proposals, feature revision
lineage, construction or maintenance records, primitive/Q/beta deltas, option
execution events, separated reward terms, admission reports, and the declared caps. The bytes returned by
`parameter_update` are exactly the canonical JSON encoding of the dictionary
returned by `observe`.

## Controls and claim boundary

`options_enabled=False` disables option initiation. It leaves feature
discovery, subproblem construction, option learning, admission requests, and
all tariffs enabled. This is the primitive-only control.

`reward_mode="oblivious"` removes the environmental reward term only from the
option cumulant. It leaves the environmental reward signal, feature discovery,
beta learning, initiation requirements, primitive learner, accounting, and
admission path unchanged. Both modes remain factual Track A runs because they
are single-pass and replay-free. The oblivious mode is explicitly ineligible
for a stronger reward-respecting claim. Historical comparator fixture labels
remain historical and do not redefine the runtime track taxonomy.

## Independent evidence reconstruction

The verifier can reconstruct each transition without trusting a finished
summary:

1. Match the one-based audit update and primitive action to the causal
   observation/action/feedback frames and source event ID.
2. Recompute all four capacity tariffs from the declared actions, width, and
   fixed caps; cumulatively sum them and match `work_by_class` and
   `WorkAccounting.work_items`.
3. Start from zero primitive, Q, and beta tables. Recompute the primitive,
   Q, and full-state beta deltas from the public transitions, compare the
   recorded deltas, and count their parameter touches.
4. Recompute the reward-respecting or oblivious cumulant from the raw reward,
   attainment flag, and stopping bonus. Check that only the original reward
   term differs between paired reward-mode runs.
5. Link feature revision parents, subproblem IDs, option IDs, and invocation
   IDs. Reconstruct option durations from initiate/action/continue/terminate
   events and verify the reported termination reason.
6. Canonically serialize each resulting bounded snapshot and update. Check the
   before/after SHA-256 linkage, serialized size, retained allocation report bounds and consistency,
   exact `parameter_update` bytes, zero replay, and all hard caps.

A meaningful live diagnostic should use the ordinary ordered adapter/session
protocol and an environment with public position and context, plus a private
rewarding goal. It should repeat the same learned feature activation in a new
full-state context, prove that the first sample does not terminate, then prove
termination after minimum support. At least two completed invocations must
have different durations. The goal should not end the environment, so learned
termination is distinguishable from environment termination. The paired
primitive-only and reward-oblivious controls use the same seeds, environment dynamics, budgets, and limits.
Realized transitions can differ after a control changes action choice. The environment supplies consequences and
reward only; it must not inject a policy, option label, target, or termination
answer.
