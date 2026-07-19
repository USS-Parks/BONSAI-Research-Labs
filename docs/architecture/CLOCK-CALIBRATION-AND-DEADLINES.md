# Clock calibration and deadline basis

BM-02 makes process-local monotonic time the sole authority for durations, deadlines, and call overhead. Wall-clock readings are optional comparison evidence for drift, regression, and suspend-or-pause annotation; they never order events or replace monotonic elapsed time.

Calibration records the minimum positive observed monotonic increment as effective resolution and the maximum bracketed probe cost as call overhead. The versioned report fails when either exceeds its declared policy or when a clock regresses. A wall/monotonic gap large enough to suggest suspend or a long scheduling pause is retained explicitly without fabricating elapsed work.

System probes share one process-local monotonic epoch. Cross-process timestamp comparison remains unqualified until a platform backend supplies and calibrates a documented common basis. CI fixtures therefore prove portable report semantics and safe hosted-run bounds, not physical-host precision or cross-process equivalence.
