# Subproblem metrics

BK-06 records attained feature and intensity, original reward, stopping bonus/value, initiation/termination, learning progress, success, and cost. Reward-respecting subproblems are aligned only when success matches a positive original reward. Reward-oblivious subproblems may succeed when original reward is non-positive and are never reclassified as reward-respecting.

The committed fixture at `fixtures/subproblem-metrics/v1/expected-outcomes.json` distinguishes those cases and retains feature lineage on every row.
