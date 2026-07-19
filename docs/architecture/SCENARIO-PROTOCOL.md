# Scenario protocol and deterministic fixtures

BE-01 defines a versioned scenario identity, seed, horizon, discrete action space, observation width, big-world cardinality, and ordered change points. A repository-owned xorshift generator plus canonical JSON produces stable semantic streams and stream hashes without relying on platform random-number implementations.

Public transitions contain only observations, allowed and chosen actions, reward, termination, and change-point markers. Latent state, target action, and the big-world diagnostic token live in a separately hashed observer-only channel. If any diagnostic truth reaches the agent, the run is forced to Track D. The M1 fixture is diagnostic infrastructure, not scientific evidence of agent quality.
