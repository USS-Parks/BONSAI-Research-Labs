# Optional Gymnasium environment adapter

BX-11 uses Gymnasium 1.3.0, pinned as the `gymnasium` optional extra. Core runs and imports do not require it. Install it only when needed with `uv sync --frozen --extra gymnasium`; default `uv sync --frozen` retains the core dependency set.

The supported environment is `FrozenLake-v1`, fixed `4x4` map, `is_slippery=false`, no rendering, four discrete actions and one discrete public observation coordinate. Episode length is an explicit bounded configuration value. The adapter accepts only the declared configuration fields and package version. It projects only the API observation, reward and independent termination/truncation flags; `info`, transition probabilities, map/model internals and future actions are never sent to the learner.

Reuse: `SessionAdapter` supplies the same framed lifecycle and observation/action protocol as the existing causal environment. Gymnasium supplies the actual environment and TimeLimit behavior. The generic supervisor preserves both end flags when they coincide. The scientific verifier keeps historical causal-source semantics and supports the pinned new environment through an explicit source/configuration contract. Both learners continue using the same runner and report path.

Sources reviewed 2026-09-06: [Gymnasium 1.3.0 distribution](https://pypi.org/project/gymnasium/1.3.0/), [versioned FrozenLake source](https://github.com/Farama-Foundation/Gymnasium/blob/v1.3.0/gymnasium/envs/toy_text/frozen_lake.py), [MIT license](https://github.com/Farama-Foundation/Gymnasium/blob/v1.3.0/LICENSE), and [official environment semantics](https://gymnasium.farama.org/environments/toy_text/frozen_lake/). The MIT license is permissive; the dependency retains its upstream notices. No upstream implementation source was copied into the adapter.

Acceptance requires direct-versus-framed traces for identical seeds/actions, explicit truncation and simultaneous end-flag cases, regular governed runs with both learners, source/dependency provenance, default-core installation checks, full local gates and hosted CI. Diagnostic integration evidence establishes no learning-quality, physical-host or energy claim.
