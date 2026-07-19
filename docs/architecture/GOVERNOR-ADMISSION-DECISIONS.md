# Governor admission decisions

BQ-02 derives one outcome from immutable policy, typed request, and BQ-01 scope projections. Precedence is fixed: unavailable required measurement rejects, hard overage rejects, rolling soft overage defers to the supplied next release, non-rolling soft overage throttles, and an entirely in-budget request admits.

The caller cannot supply or override an outcome or reason. Projections are sorted by stable limit ID before serialization. The decision body contains the policy identity/version/hash, request, all observed projections, outcome-specific action, and stable reason code. Its canonical JSON SHA-256 and complete evidence bytes are deterministic across operating systems.

Missing hard-counter input produces `HARD_COUNTER_UNAVAILABLE` before work. BQ-02 does not commit an admitted charge, terminate a process, or implement the violation lifecycle; those are BQ-03/BQ-04 responsibilities.
