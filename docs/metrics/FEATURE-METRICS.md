# Feature metrics

BK-05 derives birth/age/activation/retirement, novelty, redundancy, consumers, marginal contribution, useful lineage, utility per byte/work, churn, dormancy, and obsolete-protection counts from ordered feature lineage and activation traces.

Classification uses only machine lineage/activation/consumer/utility evidence. Human-semantic labels are never consulted. Duplicate representation identities mark redundancy when marginal utility is non-positive. Useful lineage requires positive marginal utility and at least one consumer. Dormancy covers never-activated or long-idle zero-consumer features. Obsolete protection covers retired features that still have consumers.

The committed corpus at `fixtures/feature-metrics/v1/expected-outcomes.json` freezes useful, redundant, dormant, and obsolete-protected identities with exact metric numerators.
