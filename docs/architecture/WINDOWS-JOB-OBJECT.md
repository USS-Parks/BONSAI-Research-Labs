# Windows Job Object measurement and enforcement

BM-05 and BM-06 define the Windows backend. Portable process-tree counters are used when the host is Windows. Job Object attach is not bound in this crate because the workspace forbids `unsafe_code`; a physical Windows host probe may later supply Job Object accounting records.

Capability detection never invents a live Job Object pass. Off-Windows hosts record `NOT_WINDOWS_HOST`. Child-process escape is a first-class reconciliation failure. Supported hard-limit fixtures record bounded overshoot and termination; unsupported limits are declared and cannot silently become soft.

This is not a security sandbox. D-21 remains in force.
