# Utility estimator hierarchy

BK-10 implements the D-20 hierarchy: exact leave-one-out counterfactuals, matched ablation attribution, consumer credit, and influence proxies. Each row is labeled with its highest honest tier and records cost as `utility_per_cost`.

Exact and matched estimators may be C3-eligible when available. Consumer credit and proxy-only rows are never C3-eligible. When a proxy is present beside an exact diagnostic, a sign disagreement or confidence below the declared threshold forces indeterminate utility rather than a numeric override.

The committed fixture at `fixtures/utility-metrics/v1/expected-outcomes.json` covers exact-positive, proxy sign-error, proxy confidence-failure, and proxy-only cases.
