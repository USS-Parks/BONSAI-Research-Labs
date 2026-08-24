# Linux cgroup v2 measurement and enforcement

BM-09 reads cgroup v2 `cpu.stat`, `memory.current`, `pids.current`, and controller lists from the current process tree. Nested PID coverage is required for reconciliation. cgroup v1 is not a release requirement.

BM-10 applies CPU, memory, I/O, and PID limits only when a child cgroup can be created. Undelegated controllers fail closed with `CGROUP_CONTROLLER_NOT_DELEGATED` before Track A. Hosted container evidence is not physical-host acceptance.
