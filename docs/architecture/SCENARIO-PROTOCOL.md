# Scenario protocol and deterministic fixtures

BE-01 defines a versioned scenario identity, seed, horizon, discrete action space, observation width, big-world cardinality, and ordered change points. A repository-owned xorshift generator plus canonical JSON produces stable semantic streams and stream hashes without relying on platform random-number implementations.

Public transitions contain only observations, allowed and chosen actions, reward, termination, and change-point markers. Latent state, target action, and the big-world diagnostic token live in a separately hashed observer-only channel. If any diagnostic truth reaches the agent, the run is forced to Track D. The M1 fixture is diagnostic infrastructure, not scientific evidence of agent quality.


## BX-04 online causal diagnostic

ScenarioSession reuses ScenarioSpec and the repository xorshift generator while preserving semantic_stream byte-for-byte for archived schedule-based fixtures. Its distinct dynamics identity is bonsai.causal-ring/v1; the stream identity binds that mode, the complete spec, and the episode seed supplied at reset.

Reset returns a current observation. At step i, action a moves the private ring state to (state + a + 1) modulo big_world_size. The private goal is deterministically seeded; configured change points move it using the same seeded generator. Reaching the goal terminates with reward one. A non-goal transition at the horizon truncates with reward zero. Terminal wins when both conditions would occur. Subsequent actions fail until reset. Queries return the cached observation without advancing randomness.

Public observations contain stream identity, current step, observation values, and allowed actions. Completed transitions add the chosen action, reward, and separate terminated/truncated flags. Goal and latent state remain in the separate observer-only diagnostic type. Public observations are seeded projections of current state; they are not diagnostic truth or future schedules. The learner boundary has no diagnostic query. Private diagnostic fixtures are retained separately from public traces.

Session shape is bounded before allocation: at most 256 actions, 256 observation values, and 4096 change points. It retains current state rather than an episode transcript. Seed, ordering, action, reset, and finished-episode failures use fixed codes. Invalid scenario actions do not advance state.

## Executable environment boundary

The Python environment_adapter module runs as a child behind the existing bounded length framing and AdapterFrame Protobuf protocol. It loads one size-bounded spec file; Configure must match the exact file SHA-256 and accepted capability fingerprint. Start/Handshake, Configure/Ack, Reset/Ack, and Stop/Stopped preserve epoch 1 minor 0. The environment reads its own configuration, writes no telemetry files, and exposes no network operation.

After each Reset/Ack, the supervisor sends one Step of declared input type bonsai.environment.observe/v1 with empty input. The returned StepResult.action holds a CausalObservation. Later Steps use bonsai.environment.action/v1 with a CausalAction payload; results contain CausalTransition. These are additive payload message types in the existing proto file, not a replacement envelope or transport. Every payload remains SHA-256 bound. The outer Step index counts requests starting with observation query zero; CausalAction.step independently counts environment actions starting at zero.

An invalid request produces a bounded ProtocolError and exits the adapter with status two. Framing is capped at 64 KiB; the supervisor retains responsibility for I/O deadlines and process cleanup. Tests exchange actions only after receiving the preceding observation/result, verify live causal divergence, reject malformed/order/post-episode traffic, and require bounded process exit. No hostile-code sandbox claim is added.

The checked-in Python .py and .pyi files are generated from the existing Cargo-locked protoc-bin-vendored 3.2.0 package. Only that generated namespace is excluded from handwritten lint/type analysis; check_python_protocol.py regenerates and compares both files byte-for-byte. The test requires the Cargo workspace's locked compiler cache. Rust consumes the real captured Python transcript through the existing AdapterProtocolMachine and verifies exact re-encoding. Archived adapter and semantic-stream fixtures remain regression gates.

The local WSL runtime uses target/linux-venv because a Windows virtual environment cannot supply Linux Protobuf binaries. It is a small reusable environment in the canonical checkout, installed from uv.lock; Windows .venv is preserved. Hosted tests still exercise the pinned Python 3.12 matrix. This prompt establishes a causal boundary, not a complete governed experiment or scientific claim.
