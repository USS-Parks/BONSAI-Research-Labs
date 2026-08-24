# Metric reproducibility and performance gate

BK-14 recomputes a derived table twice and requires byte-identical semantics inside a declared live-row envelope. The observer path has no agent access. Dropping required evidence to shrink memory is rejected.
