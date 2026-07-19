# Governor violation lifecycle

BQ-03 records warnings, degradation, hard violations, termination, recovery, and the final verdict as one append-only state sequence. Legal edges are explicit. A terminal state cannot re-enter the lifecycle, and a hard violation cannot yield claim-eligible evidence.

Soft-only completion is retained as `soft_degraded`. A hard violation passes through termination and settles as `hard_violated`. A fault from any active boundary appends `failed` then `recovered`; recovery closes evidence only and never resumes governed work. Every outcome contains one terminal state, contiguous ordinals, stable reason codes, and a self-validating failure bundle.

BQ-03 is a deterministic lifecycle and evidence authority. It does not monitor processes, enforce an operating-system cap, or supervise a reference agent; BQ-04 integrates the basic portable loop.
