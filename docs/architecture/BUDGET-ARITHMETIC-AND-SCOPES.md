# Budget arithmetic and scopes

BQ-01 implements typed external accounting for one counter/unit/work-class key across per-event, per-step, rolling-window, and lifetime scopes. A request is projected through every matching scope before mutation. Equality with a soft or hard threshold remains inside that threshold; only a strictly greater projection exceeds it.

All additions are checked. Overflow and monotonic-time regression fail closed. Per-event and per-step resets do not alter lifetime totals. Rolling charges expire exactly when their age reaches the declared window duration. Committing one admitted request updates all four accounting views exactly once.

Missing measurements produce `measurement_unavailable` with no consumed or projected numeric value. They are never treated as zero. BQ-02 will map these typed projections to deterministic admit/defer/throttle/reject decisions; BQ-01 does not choose an outcome or enforce an OS limit.
