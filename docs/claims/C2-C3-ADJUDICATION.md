# C2 and C3 adjudication

BV-05 requires C1 before C2, and C2 before C3. C2 is continual adaptation across first and return phases with at least 20 paired seeds and a Holm-rejected positive effect. Final-score-only adaptation fails. Missing phases or seed counts stay indeterminate.

C3 requires positive marginal abstraction utility from exact leave-one-out or matched ablation. Proxy-only utility and comparator-track leakage cannot pass. Missing utility stays indeterminate.

The committed fixture at `fixtures/c2-c3-adjudication/v1/expected-outcomes.json` covers pass, fail, and indeterminate for every prerequisite.
