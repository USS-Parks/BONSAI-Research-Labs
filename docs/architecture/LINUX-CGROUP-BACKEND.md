# Linux cgroup v2 measurement and enforcement

BM-09 reads cgroup v2 `cpu.stat`, `memory.current`, `pids.current`, and controller lists from the current process tree. Nested PID coverage is required for reconciliation. cgroup v1 is not a release requirement.

BM-10 does not write `cpu.max`, `memory.max`, `io.max`, or `pids.max`. Creating a child cgroup is a delegation probe, not a hard limit. Limits stay `NoPermission` or `Unsupported` until a writer exists. Track A stays closed. Hosted container evidence is not physical-host acceptance.
