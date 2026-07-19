# Basic supervised budget loop

BQ-04 governs one primitive step loop with required per-step and lifetime CPU-time, memory, storage, latency, and work-item charges. All five portable counters must be available. Work-item overages and unavailable required counters reject before the agent closure is called. Measured post-step overages terminate the loop and retain all earlier admission and usage evidence.

An under-budget report is marked `c1_budget_eligible`; that field is only the budget prerequisite consumed later by BV-04, not a C1 verdict. Any denial, termination, unavailable counter, or lifecycle failure makes it false. BQ-04 applies portable monitor/deny/terminate semantics and does not claim platform-specific kernel hard caps or bounded physical overshoot.
