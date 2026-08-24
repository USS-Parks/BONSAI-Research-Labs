# Continual-learning metrics

BK-04 separates retention, adaptation, forgetting, and plasticity loss. Forgetting is the drop on the first task after an intervening task. Plasticity loss is the shortfall of intervening-task performance versus that phase's attainable score. The two quantities are independently manipulable and must not be collapsed into a single "failed to retain" score.

The engine also reports transfer against an optional no-prior baseline, interference (the same first-task drop), relearning steps until competency returns, phase divergence, and exact-age performance curves. Missing transfer baselines or unrecovered return phases are unavailable, never numeric zero.

Synthetic fixtures at `fixtures/continual-metrics/v1/expected-outcomes.json` freeze a forget-and-learn case, a retain-but-rigid case, and a retain-and-adapt case.
