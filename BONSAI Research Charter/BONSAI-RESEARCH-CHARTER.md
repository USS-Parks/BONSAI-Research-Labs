# BONSAI Research Charter

**Benchmark for Online, Nonstationary, Single-pass Agent Intelligence**

Version: 0.1  
Date: 2026-07-18  
Status: Approved v0.1 on 2026-07-18; implementation not authorized  
Authoritative working location: `C:\Users\17076\Documents\Reinforcement Learning Project\BONSAI Research Charter`

## 1. Charter purpose

BONSAI will be a comprehensive, algorithm-neutral measurement and resource-governance layer for determining whether an OaK-style cycle of discovery is producing useful, increasingly abstract cognition from runtime experience.

The instrument must distinguish genuine progress from uncontrolled feature growth, option proliferation, model accumulation, benchmark overfitting, hidden replay, or unreported computational expansion.

The charter defines:

- the scientific questions BONSAI must make falsifiable;
- the objects and events it must observe;
- the resource constraints it must enforce;
- the comparisons and ablations it must support;
- the claims it may and may not justify;
- the cross-platform and reproducibility obligations inherited by the later implementation PSPR.

This charter does not select Oak Lab's learning algorithms, implement OaK, or authorize software construction.

## 2. Scientific thesis

The central BONSAI question is:

> Under fixed and explicitly measured compute, memory, storage, latency, and energy constraints, can a single-pass online agent grow reusable state and temporal abstractions whose downstream planning and behavioral benefit exceeds their continuing acquisition and maintenance cost?

An OaK-style discovery cycle is working only if its learned features, subproblems, options, and option models demonstrably improve downstream behavior or adaptation under controlled resource budgets.

Internal activity is not sufficient evidence. More features, more options, more parameter updates, deeper lineages, or longer runtime are not successes unless their marginal utility is established.

## 3. Research basis

BONSAI is motivated by the following public OaK commitments:

1. **Domain generality** — the essential agent design should not contain knowledge specific to a particular world.
2. **Runtime experience** — important learning, planning, modeling, and abstraction discovery occur online during the agent's lifetime.
3. **Open-ended abstraction** — sophistication may grow with experience, limited principally by computational resources.
4. **Big-world operation** — the world is larger and more complex than the agent; learned state, policies, values, and models are necessarily approximate.
5. **Single-pass learning** — the canonical Oak direction learns directly from experience without storing or replaying the stream.
6. **Temporal abstraction** — feature-attainment subproblems are solved to produce options; option consequences become learned knowledge used in planning.
7. **Downstream utility credit** — consumers of abstractions evaluate and shape the abstractions on which they depend.

The public architecture remains schematic. Reliable nonlinear continual learning and runtime discovery of new state features are explicitly unresolved. BONSAI exists to measure candidate solutions without prematurely declaring those problems solved.

## 4. BONSAI's object of observation

The target discovery cycle is:

1. Runtime experience produces observations, actions, and reward.
2. Perception constructs and revises state features.
3. Selected features define reward-respecting feature-attainment subproblems.
4. Solving a subproblem produces an option and associated value estimates.
5. Predicting an option's consequences produces an option model, treated as knowledge.
6. Planning with action and option models updates values and behavior.
7. Downstream consumers return evidence of utility to features, subproblems, options, and models.
8. Resource governance retains, deprioritizes, replaces, or removes artifacts.

BONSAI must observe both the forward production flow and the slower backward flow of utility credit.

## 5. Scope

### 5.1 In scope

- A platform-neutral event and telemetry contract for online agents.
- Stable identities and lifecycle records for features, subproblems, options, models, planners, policies, and value functions.
- Single-transition, batch-size-one measurement.
- Canonical no-storage/no-replay evaluation and explicitly labeled comparator tracks.
- Measurement of learning, adaptation, abstraction, planning benefit, interference, plasticity, and resource consumption.
- Budget enforcement for compute, memory, persistent storage, latency, update work, and energy where measurable.
- Event-driven and dense-update scheduler comparisons.
- Deterministic experiment manifests, artifact provenance, and replayable telemetry analysis without granting the learning agent replay access.
- Windows, macOS, and Linux support.
- Reference environments and adapters sufficient to exercise the discovery cycle.
- Component-level, cycle-level, and end-to-end ablations.
- Honest capability claims and machine-verifiable result bundles.

### 5.2 Explicitly excluded from the first implementation scope

- Claiming to reproduce unpublished Oak Lab algorithms.
- Building a general-purpose superintelligent agent.
- Selecting a single mandatory neural architecture.
- Requiring neuromorphic or spiking hardware.
- Embodied robotics hardware as a baseline dependency.
- Cloud services as a requirement for core experiments.
- Human-label supervision in the canonical experiential track.
- Treating a simulator score as proof of open-ended intelligence.
- Production deployment, autonomous real-world control, or safety certification.

### 5.3 Parked for later authorization

- Hardware energy probes and laboratory-grade power instrumentation.
- Cluster-scale or distributed learners.
- Mobile and embedded deployment targets.
- External benchmark submission services.
- Live collaboration with or publication on behalf of Oak Lab.

## 6. Governing principles

### 6.1 Observer independence

BONSAI measures and constrains the agent but does not determine the agent's learning algorithm. Instrumentation interfaces must admit linear, neural, symbolic, hybrid, dense, sparse, synchronous, and event-driven implementations.

### 6.2 One stream, two audiences

The agent receives the authorized experience stream. BONSAI may retain telemetry for scientific audit, but retained telemetry must be isolated from the agent and must not become an undeclared replay buffer.

### 6.3 Resource truthfulness

Every reported result must state which resources were measured directly, estimated, unavailable, or excluded. Missing energy telemetry must never be represented as zero energy.

### 6.4 Fixed-budget comparison

Algorithm comparisons must be made under declared and enforceable budgets. Equal wall time alone is insufficient when hardware, parallelism, or measurement overhead differs.

### 6.5 Utility before ontology

BONSAI will not require learned features to match human concepts or labels. A feature earns standing through measurable downstream use, predictive contribution, control contribution, planning gain, adaptation benefit, or justified exploratory value.

### 6.6 Lineage and accountability

Every mutable cognitive artifact must have a stable identity, provenance, parents where applicable, consumers, resource cost, utility history, and terminal disposition.

### 6.7 Falsifiability

Every high-level claim must map to an operational metric, controlled comparison, failure threshold, and stored evidence artifact.

### 6.8 Cross-platform equivalence

Windows, macOS, and Linux runs must share experiment semantics and result schemas. Platform-specific measurement backends are permitted, but unsupported counters and differences in precision must be declared.

## 7. Canonical evaluation tracks

### Track A — Strict experiential

- Batch size one.
- One authorized encounter with each transition.
- No replay buffer.
- No offline training phase.
- No human labels or domain-specific feature targets.
- Fixed per-step and lifetime resource budgets.

This is the canonical BONSAI track.

### Track B — Bounded replay comparator

- Replay is allowed only within a declared byte and transition budget.
- Replay work counts toward compute, memory, storage, latency, and energy totals.
- Results must never be merged with Track A.

### Track C — Dense-update comparator

- All eligible components may be updated on every step.
- Used to compare event-driven scheduling with dense computation under matched budgets and matched semantic inputs.

### Track D — Oracle and diagnostic controls

- May use privileged state, known dynamics, fixed features, or designed options.
- Used to localize failure and establish ceilings.
- Not eligible for domain-general or experiential capability claims.

## 8. Measurement domains

### 8.1 Primary behavior

- cumulative reward and reward rate;
- regret where a defensible comparator exists;
- area under the lifelong learning curve;
- time or experience to reach defined competency thresholds;
- recovery time and recovery cost after change;
- worst-window performance, not only final or mean performance.

### 8.2 Continual learning

- retention of still-relevant competence;
- adaptation to newly relevant structure;
- loss of plasticity separated from catastrophic forgetting;
- forward transfer, backward interference, and relearning time;
- update stability and divergence incidence;
- performance as a function of agent age.

### 8.3 Features

- birth, parentage, age, activation, and retirement;
- novelty relative to the active population;
- redundancy and conditional contribution;
- number and importance of downstream consumers;
- marginal behavioral, predictive, option-learning, model-learning, and planning utility;
- useful lineage depth and breadth;
- utility per byte and per unit of update work;
- churn, dormancy, and protected-but-obsolete features.

### 8.4 Subproblems and options

- feature and intensity used to pose each subproblem;
- option success, duration, cumulative reward, stopping-state value, and termination distribution;
- option distinctiveness and redundancy;
- controllability and reliability across starting states;
- whether an option participates in a maximizing or otherwise consequential planning backup;
- marginal planning and behavior gain attributable to the option;
- acquisition and maintenance cost.

### 8.5 Models and knowledge

- one-step and option-horizon prediction error;
- reward prediction error;
- stopping-state calibration;
- error as a function of temporal jump length;
- model drift and recovery;
- uncertainty or confidence where supplied;
- marginal planning gain and harmful-planning incidence;
- stability of model semantics as underlying representations change.

### 8.6 Planning

- planning updates, states considered, options considered, and search-control policy;
- value improvement per planning operation;
- realized agreement between planned gain and subsequent experience;
- planning depth in primitive-time equivalents;
- option-induced reduction in backups or decision latency;
- model exploitation failures;
- computation devoted to plans that never influence behavior.

### 8.7 Discovery-cycle health

- forward production rate at every stage;
- backward utility-credit latency and magnitude;
- survival probability conditioned on demonstrated utility;
- useful abstraction generations completed;
- cycle bottleneck location over time;
- growth rate of useful capability versus total cognitive population;
- maintenance cost versus marginal benefit;
- collapse, runaway growth, cycling, and ossification indicators.

### 8.8 Resources

- process and system wall time;
- monotonic elapsed time;
- CPU time and utilization;
- accelerator time and utilization where available;
- allocated and resident memory;
- agent-accessible persistent storage;
- bytes retained by the scientific observer;
- environment steps and agent updates;
- parameter touches, update events, and model/planning operations;
- estimated operations using a declared method;
- energy measured through supported platform or hardware backends;
- instrumentation overhead and sampling error.

## 9. Resource-governance requirements

BONSAI must support enforceable limits at four scopes:

1. **Per event** — maximum work triggered by one experience or internal event.
2. **Per environment step** — maximum learning, modeling, planning, and curation work before the next action deadline.
3. **Rolling window** — protection against bursts that satisfy averages but violate real-time operation.
4. **Lifetime** — total compute, memory growth, storage, and energy envelope.

The governor must be able to:

- admit, defer, throttle, or reject work;
- allocate budgets among acting, learning, feature generation, option learning, model learning, planning, and curation;
- record why a work item ran or did not run;
- prevent telemetry retention from becoming agent-accessible replay;
- distinguish measured work from estimated work;
- terminate or fail closed when a hard budget cannot be enforced;
- expose soft-budget degradation separately from hard-budget violations;
- preserve enough evidence to reproduce every governance decision.

Resource policy must be external to the evaluated agent for canonical comparisons. Learned internal scheduling may be evaluated, but the external governor remains the enforcement authority.

## 10. Event-driven evaluation contract

Because Oak Lab's detailed event-driven design is not yet public, BONSAI will define event-driven behavior operationally rather than neurologically.

An event-driven candidate must declare:

- its event types and event producers;
- eligibility rules for waking each component;
- work suppressed relative to the dense comparator;
- state retained between events;
- ordering, concurrency, and deadline semantics;
- whether updates are exact, approximate, delayed, or dropped;
- the method used to attribute resource cost and downstream benefit.

BONSAI must not assume that event-driven means spiking, asynchronous sensing, sparse activations, or delta propagation. Those may be evaluated as implementations under the common contract.

## 11. Reference scenario families

The implementation PSPR must refine at least these scenario families:

1. **Stable dynamics, changing values** — tests whether learned models and planning support rapid revaluation.
2. **Observation aliasing** — a stationary world appears nonstationary because the agent's state is insufficient.
3. **Late-arriving latent factors** — initially adequate representations become inadequate, requiring new features.
4. **Long-life plasticity** — difficulty remains controlled while the learner ages across many changes.
5. **Temporal joints** — useful sustained behaviors exist and option models should reduce planning work.
6. **Distractor abundance** — many salient or predictable features have little downstream utility.
7. **Resource shock** — available compute, memory, or action latency changes during runtime.
8. **Model mismatch** — temporally extended models become biased or stale and can harm planning.
9. **Recursive reuse** — useful higher-level features require reuse of previously learned features or options.
10. **Noisy single-pass stream** — the agent must learn from noisy experience without curation or replay.

Each family must include at least one diagnostic setting in which the correct causal explanation is known and at least one big-world setting in which only operational performance is available.

## 12. Mandatory ablations

At minimum, BONSAI must support comparisons that remove or replace:

- learned features with fixed features;
- feature utility with random retention or age-based retention;
- reward-respecting subtasks with reward-oblivious alternatives;
- learned options with primitive actions only;
- option models with primitive transition models only;
- planning with reactive/model-free control;
- backward utility credit with local-only utility;
- event-driven scheduling with dense updating;
- no replay with bounded replay;
- continual feature replacement with a frozen network;
- resource governance with unconstrained growth.

An end-to-end result without component ablations cannot establish that the discovery cycle caused the outcome.

## 13. Claim ladder

BONSAI will use progressively stronger claims:

### C0 — Instrumented

Events, artifacts, and resources are recorded with valid provenance.

### C1 — Resource compliant

The run stays within its declared enforceable budgets.

### C2 — Continually adaptive

The agent retains measurable learning ability over the required lifetime and responds to relevant change.

### C3 — Useful abstractions

Learned features or options have positive marginal downstream utility under controlled ablation.

### C4 — Working discovery cycle

Forward construction plus backward utility credit produces better retained abstractions than controlled alternatives.

### C5 — Resource-positive open-ended growth

Multiple generations of newly discovered abstractions continue to produce net capability gain under a fixed or explicitly scaled resource envelope.

No single benchmark score may justify C4 or C5. These claims require converging evidence across scenario families, seeds, lifetimes, and ablations.

## 14. Failure criteria

The discovery cycle is not working when any of the following persists beyond declared tolerance:

- useful behavior does not exceed primitive or fixed-feature controls;
- abstraction count rises while marginal utility approaches zero or becomes negative;
- option models increase planning error or computation without compensating gain;
- old abstractions are protected by stale credit while useful new ones are rejected;
- the feature population collapses to redundant or trivial correlates;
- adaptation degrades toward or below the linear or tabular diagnostic baseline with age;
- hidden replay, offline updates, privileged labels, or undeclared compute affect results;
- resource compliance depends on omitted or unsupported measurements;
- the run cannot be reproduced within declared statistical and platform tolerances;
- claimed open-endedness disappears when compute, memory, or population size is held fixed.

Failure is a valid and scientifically useful result. BONSAI's purpose is adjudication, not promotion.

## 15. Cross-platform obligations

The later PSPR must require:

- first-class Windows, macOS, and Linux execution paths;
- no shell-specific core semantics;
- path, newline, process, signal, and clock abstractions tested on all three platforms;
- monotonic timing for durations;
- explicit CPU architecture, accelerator, driver, compiler/runtime, and power-backend metadata;
- portable result bundles and schemas;
- deterministic seeds where the evaluated stack permits determinism;
- declared nondeterminism where it does not;
- feature detection rather than platform-name assumptions;
- graceful degradation when a metric is unavailable;
- platform equivalence tests separated from performance equivalence claims;
- CI smoke tests on all three operating-system families;
- longer acceptance runs on representative physical hosts, not CI alone.

Energy measurement will require tiered evidence because no uniform high-fidelity interface exists across all machines:

- Tier E0: energy unavailable; no energy claim permitted.
- Tier E1: software or vendor estimate with provenance and resolution disclosed.
- Tier E2: platform counter with calibration and sampling uncertainty.
- Tier E3: external power measurement with synchronized run boundaries.

## 16. Reproducibility and evidence bundle

Every reportable run must produce a machine-readable bundle containing:

- immutable experiment manifest;
- source revision and dirty-state declaration;
- platform and dependency inventory;
- environment configuration and seed schedule;
- track classification and replay declaration;
- resource policy and governor decisions;
- event schema version;
- metric definitions and units;
- raw or losslessly transformed telemetry references;
- summaries with confidence intervals or uncertainty treatment;
- failures, dropped events, unavailable counters, and instrumentation overhead;
- artifact lineage graph;
- claim-level pass, fail, or indeterminate verdicts.

An indeterminate result must remain indeterminate. Missing evidence cannot be converted into a pass.

## 17. Security and integrity boundaries

The measurement layer must assume evaluated agents and plugins can be faulty. The PSPR must address:

- schema validation and bounded inputs;
- isolation between agent-accessible state and observer-only telemetry;
- denial-of-service through event floods or artifact creation;
- tamper-evident manifests and result hashes;
- safe termination and recovery after budget violations;
- redaction of secrets and machine-specific sensitive data;
- no network requirement for canonical local runs;
- explicit authorization before external publication or transmission.

This charter does not claim adversarial sandboxing of arbitrary hostile native code unless that capability is later added explicitly to the PSPR.

## 18. Completion criteria for the BONSAI instrument

The measurement and resource-governance layer will be charter-complete only when:

1. All canonical object lifecycles and event types are represented.
2. Resource budgets are enforced and evidenced at the required scopes.
3. Strict experiential and comparator tracks cannot be confused in artifacts or reports.
4. The mandatory ablations can be run through stable component contracts.
5. At least one reference OaK/STOMP-style agent and appropriate control agents are measured end to end.
6. Claim-ladder verdicts are machine generated from versioned criteria.
7. Windows, macOS, and Linux gates pass at their prescribed evidence levels.
8. Measurement overhead is quantified and bounded.
9. Result bundles are independently auditable and reproducible.
10. Documentation states what BONSAI cannot conclude as clearly as what it can.

Instrument completion does not imply that an evaluated discovery cycle passes C4 or C5.

## 19. Settled defaults for PSPR drafting

- BONSAI is an independent observer and governor, not the OaK implementation.
- Strict single-pass, batch-size-one, no-replay operation is canonical.
- Replay and privileged information are comparator tracks only.
- Windows, macOS, and Linux are first-class targets.
- Local execution must not require cloud services.
- Measurements carry units, provenance, precision, and availability state.
- Resource budgets are externally enforceable.
- The agent cannot consume observer-retained telemetry in canonical runs.
- Utility is demonstrated through downstream effect and controlled ablation.
- Open-endedness is a strong claim requiring multigenerational, resource-positive evidence.
- The PSPR will be review-first and execution will require explicit STS authorization.

## 20. Questions intentionally deferred to PSPR review

- Primary implementation language and runtime stack.
- Process, plugin, and inter-process component boundaries.
- Exact event serialization and storage technology.
- Reference environment suite.
- First reference agents and algorithms.
- Cross-platform energy backends.
- Statistical repetition budgets and acceptance tolerances.
- Visualization and reporting interfaces.
- Packaging, release, and continuous-integration topology.
- Whether hostile-code sandboxing is required.
- Licensing and public-release strategy.
