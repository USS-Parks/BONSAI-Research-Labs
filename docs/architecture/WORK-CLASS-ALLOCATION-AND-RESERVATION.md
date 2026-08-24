# Work-class allocation and reservation v1

Status: BQ-05 authority for deterministic semantic-work partitioning.

The external governor partitions one fixed capacity across acting, learning, feature generation, option learning, model learning, planning, curation, and observer work. Reservations are hard, exclusive partitions. The allocator does not estimate scientific utility, borrow from another class, or optimize the partition.

## Policy invariants

An `AllocationPolicy` must name every allocatable class exactly once with a positive reservation. Environment work is intentionally outside this allocator. Reservation amounts must sum exactly to total capacity. The observer reservation contains a positive nested evidence-flush reservation no larger than the observer partition. Missing, duplicate, zero, overflowing, or mismatched partitions fail before allocator state exists.

Acting capacity is exclusive to acting requests. Other agent work cannot consume it, so any valid acting request within its reservation remains admissible regardless of learning/planning/curation request order. The observer partition is accessible only to observer authority. An agent request labeled as observer work is rejected with `ALLOCATION_AUTHORITY_MISMATCH` without changing counters.

Ordinary observer overhead is capped at `observer reservation - evidence flush reservation`. It receives `OBSERVER_EVIDENCE_FLUSH_RESERVATION_PROTECTED` before crossing that boundary. Evidence-flush work can then consume the protected remainder. Neither ordinary observer work nor any agent work can starve evidence finalization.

## Decisions and accounting

Each valid request receives a monotonic sequence and retains authority, class, purpose, requested amount, applicable reservation, usage before/after, outcome, and stable reason. Requests above a class reservation are deferred with `WORK_CLASS_RESERVATION_EXHAUSTED`. Unsupported environment requests and authority mismatches are rejected. Defer/reject paths leave every usage counter byte-for-byte unchanged; only admission commits a checked addition.

The committed corpus at `fixtures/work-allocation/v1/expected-outcomes.json` freezes an adversarial sequence that fills learning, attempts cross-partition and observer forgery, fills ordinary observer overhead, verifies the evidence reserve, flushes evidence, and finally consumes the still-intact acting reservation. Additional rotations flood every non-acting class before acting and prove the observer counter remains zero for agent-originated traffic.

BQ-05 allocates the policy's declared semantic-work unit. Platform-specific CPU, memory, storage, I/O, or energy enforcement remains owned by their corresponding limits/backends. Allocation success is not a utility or claim-ladder verdict.
