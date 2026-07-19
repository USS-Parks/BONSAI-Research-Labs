# BONSAI terminology, identifiers, and units v1

Status: frozen for schema epoch 1  
Owner: BONSAI schema lead  
Machine registry: `schemas/registry/terminology-v1.json`  
Exclusion: this document defines names and representations, not metric formulas

## Core execution terms

- **run** — one governed execution of an immutable experiment manifest, producing at most one canonical result bundle.
- **stream** — the ordered-by-source sequence of authorized transitions and related runtime events for a run; it does not imply a global wall-clock total order.
- **transition** — one environment interaction tuple made available once to a Track A agent: prior observation/state representation, action, reward, termination/truncation facts, and next observation/state representation as authorized by the adapter.
- **event** — a versioned, immutable telemetry envelope emitted by an authorized source with per-source sequence and causal parents.
- **work item** — one externally governable request to spend bounded work in an acting, learning, feature, option, model, planning, curation, environment, or observer class.
- **artifact** — a stable-identity cognitive or evidence object with provenance and lifecycle. Cognitive artifact types are feature, subproblem, option, model, planner, policy, and value function.
- **lineage** — directed provenance relationships among immutable artifact identities or revisions. Lineage is not permission for the agent to read observer history.
- **consumer** — an identified runtime component or artifact whose measured downstream behavior can depend on another artifact.
- **track** — one mutually exclusive evaluation class derived from enforceable runtime facts, not self-attestation.
- **bundle** — an immutable, independently verifiable run evidence package bound by hashes and versioned schemas.

## Cognitive artifact terms

- **feature** — a state representation component that can be born, revised under a new revision identity, consumed, credited, and retired.
- **subproblem** — a declared auxiliary control/prediction problem; reward-respecting feature-attainment semantics are recorded rather than inferred from its name.
- **option** — a temporally extended policy plus stopping rule.
- **model** — a versioned predictor of primitive or option consequences; “knowledge” may describe its role but is not a separate canonical artifact type in epoch 1.
- **planner** — a component that allocates planning work over models and candidate actions/options.
- **policy** — a versioned action/option distribution or selection rule.
- **value function** — a versioned predictive value estimate tied to its cumulant/reward, discount/termination, policy, and representation semantics.

Artifact lifecycle event kinds are `birth`, `revision`, `consumer_link`, `cost`, `utility`, and `disposition`. Parent provenance relations are `derived_from`, `constructed_from`, and `constrained_by`; unlike consumer links, these relations form a directed acyclic graph. `retained` and `deprioritized` are nonterminal dispositions. `replaced`, `retired`, and `removed` are terminal and require any continuation to use a new `artifact_id`.

## Evaluation tracks

- **Track A / strict experiential** — batch size one, one authorized encounter per transition, no replay/offline phase/human labels/domain feature targets, and fixed external budgets.
- **Track B / bounded replay comparator** — replay is declared and fully charged to resource budgets; results never merge with Track A.
- **Track C / dense-update comparator** — eligible components may update every step under matched semantic inputs and budgets.
- **Track D / oracle diagnostic control** — privileged state, known dynamics, fixed features, or designed options may localize failure; ineligible for experiential/domain-general claims.
- **INDETERMINATE_TRACK** — runtime facts are insufficient or contradictory; blocks C2–C5 and is not a fifth track.

## Budget scopes and decisions

Budget scopes are `per_event`, `per_step`, `rolling_window`, and `lifetime`. Canonical decision outcomes are `admit`, `defer`, `throttle`, `reject`, and `terminate`. Soft degradation and hard violation are distinct states.

Canonical work classes are `acting`, `learning`, `feature_generation`, `option_learning`, `model_learning`, `planning`, `curation`, `environment`, and `observer`.

## Claim and availability states

Claim verdicts are exactly `pass`, `fail`, `indeterminate`, and `not_run`. Completion of the instrument and evaluated-agent C0–C5 verdicts are separate.

Measurement availability is exactly:

- `measured` — directly observed through the named counter/collector;
- `estimated` — produced by a declared estimator with provenance and uncertainty;
- `unavailable` — requested but not obtainable at the required quality or privilege;
- `excluded` — intentionally outside the manifest's declared measurement set.

Unavailable, excluded, or unknown values are absent/null with an availability reason. They are never encoded as numeric zero. Zero is valid only for an available measurement whose observed value is actually zero.

## Identifier policy

Identifiers are lowercase canonical field names ending in `_id`. Their wire representation is a 128-bit UUID serialized as lowercase canonical text (`8-4-4-4-12`) in JSON and 16 bytes in binary contracts unless a later schema explicitly defines a content hash. IDs are opaque: no timestamp, host, user, track, or semantic label is inferred from their bits.

- `run_id` identifies a governed run.
- `stream_id` identifies one authorized stream within a run.
- `source_id` identifies an event producer within a run.
- `event_id` identifies an immutable event.
- `work_item_id` identifies a governable work request.
- `artifact_id` identifies a cognitive artifact across its lifecycle; revisions use a new `artifact_revision_id` while retaining the stable `artifact_id`.
- `bundle_id` is the SHA-256 content identity of the canonical bundle index and uses 32 bytes / 64 lowercase hexadecimal characters rather than UUID representation.

Identifiers never use user paths, hostnames, serial numbers, or secrets.

## Units and numeric representation

Every numeric schema field declares a canonical unit and representation. Display units may scale, but stored canonical values do not change silently.

- Monotonic timestamps and durations: unsigned integer nanoseconds (`ns`) with separately recorded effective resolution in `ns`; wall time is optional RFC 3339 UTC text and never orders concurrent events by itself.
- CPU time and action latency: unsigned integer nanoseconds (`ns`).
- Memory, storage, event size, and observer output: unsigned integer bytes (`B`); human displays use IEC `KiB`, `MiB`, `GiB` (powers of 1024).
- Counts (steps, events, updates, touches, calls, backups, artifacts): unsigned integer count (`1`).
- Energy: integer microjoules (`uJ`) when measured or estimated; provenance, tier, resolution, and uncertainty are mandatory. Displays may use joules (`J`).
- Power: integer microwatts (`uW`) when available; displays may use watts (`W`).
- Ratios, probabilities, rates normalized by like units, and utilization: finite IEEE-754 binary64 with unit `1`; range and estimator semantics belong to later schemas/metric specifications.
- Rewards and values: finite IEEE-754 binary64 with declared environment reward unit; `reward_unit_id` is required because no universal physical reward unit exists.
- Estimated operations: unsigned integer count (`1`) plus estimator identifier/version; never labeled FLOPs without a declared FLOP estimator.
- Artifact lifecycle sequence: unsigned integer count (`1`), beginning at one and increasing exactly once per artifact event.
- Artifact resource cost: unsigned integer amount in the named canonical counter unit, with availability and estimator provenance where applicable.
- Artifact utility estimate: finite binary64 in the named metric unit, with availability, evidence-event provenance, and estimator identity where applicable.

NaN and infinities are invalid in canonical JSON. Precision, resolution, estimator, uncertainty, and availability accompany values where applicable.

## Naming rules

Canonical names are lowercase snake_case in schemas and lowercase space-separated terms in prose. An alias may resolve to only one canonical term. The v1 machine registry rejects duplicate canonical names, duplicate aliases, aliases that collide with another canonical name, ambiguous definitions, and numeric fields lacking unit or representation.
