# BRDC-1 option and model stages

BE-06 solves a successful Track A subproblem into an option (policy plus termination) and learns option consequences online. Each model update consumes one transition and retains no replay. Offline multi-pass convergence is excluded.

A known tabular fixture reaches the declared error tolerance after one pass of identical transitions. A contradictory one-pass fixture reports `MODEL_TOLERANCE_FAILED` instead of claiming convergence.

The committed fixture at `fixtures/brdc1-options/v1/expected-outcomes.json` records both outcomes.
