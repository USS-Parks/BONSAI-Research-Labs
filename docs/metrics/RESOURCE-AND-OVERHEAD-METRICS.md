# Resource and overhead metrics

BK-03 emits one sorted row per CPU, wall, accelerator, memory, storage, I/O, work, or energy counter. Each row retains its platform, unit, semantic scope, measured total or unavailable reason, budget headroom, violations, calibration error, and dependent-claim readiness. Rows are never aggregated across platforms or unlike semantic scopes.

Paired instrumentation evidence records raw throughput and p95 action latency plus the upper bound of each paired 95% confidence interval. D-11 passes only when the throughput-overhead upper bound is at most 5% and the latency-overhead upper bound is at most 10%. A confidence bound below its own point estimate is rejected as contradictory evidence.
