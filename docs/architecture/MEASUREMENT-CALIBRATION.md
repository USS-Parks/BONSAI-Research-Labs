# Measurement calibration harness

BM-04 records each controlled workload counter with expected value, observed value, absolute and parts-per-million error, effective resolution, tolerance, coverage, platform qualification, and whether dependent claims may use it. A workload report also records the observer's own wall-time cost.

Only `measured` counters with complete values/resolution and error inside the declared tolerance are claim-ready. `unavailable`, `unstable`, and `error` counters carry no numeric value and force a failed calibration verdict for any dependent claim. Missing evidence never becomes zero.

The portable harness supplies exact externally charged environment-step and work-item loads and a live single-thread CPU workload compared with process-tree accumulated CPU time. BM-03 supplies byte-exact storage calibration. Process I/O remains platform- and cache-qualified; allocation/RSS is snapshot evidence rather than portable committed-memory equivalence. Later platform and energy prompts must qualify stronger counters independently.

This gate demonstrates hosted-OS calibration semantics and local live counter behavior. It does not establish physical-host energy accuracy, hard resource enforcement, or the final D-11 instrumentation-overhead acceptance result.
