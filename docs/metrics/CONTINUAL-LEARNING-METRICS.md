# Continual-learning metrics

BK-04 derives retention, forgetting, adaptation, plasticity loss, transfer, interference, relearning, divergence, and exact-age performance curves from ordered multi-task traces.

Retain probes measure retention and forgetting against prior train baselines. Adapt phases measure adaptation and plasticity loss independently, so a trajectory may forget while remaining plastic, or lose plasticity while retaining prior competence. Transfer is the signed gap between adapt and train means. Interference mirrors retain drops. Relearning is the ratio of relearn performance to the original train baseline. Divergence is the lifetime performance range. Missing phase families are explicit detail codes, never numeric zero.

The committed corpus at `fixtures/continual-metrics/v1/expected-outcomes.json` freezes one forgetting-heavy and one plasticity-loss-heavy synthetic trajectory with distinct metric signs.
