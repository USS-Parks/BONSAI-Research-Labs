# Scheduler matched-budget conformance

BQ-12 compares dense and event-driven traces only when stream identity, seed, budget, and event count match. Unmatched work definitions fail closed. Behavior and resource deltas are reported with the charged-work difference; wall-time-only comparison is rejected.
