# macOS measurement and monitor/terminate enforcement

BM-07 collects portable CPU, resident memory, I/O, and process-tree evidence through public interfaces. Thermal counters stay `no_permission` unless a privileged collector is authorized. Private APIs are excluded.

BM-08 does not claim Job Object or cgroup parity. Enforceable controls are monitor/terminate-only. Violation fixtures record measured overshoot and process-tree termination. Missing Apple-silicon physical-host calibration is an honest capability gap, not a numeric pass.
