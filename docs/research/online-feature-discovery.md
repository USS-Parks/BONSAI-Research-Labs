# Online feature discovery (BX-12)

The executable grammar is a bounded sum of observed-coordinate equality indicators. Each feature stores a coordinate, an observed integer value, and a running reward coefficient (integer sum/count). Candidates arise only after repeated exposure with positive observed reward sum. This is a proposal heuristic, not evidence of downstream utility. No semantic label, supplied representation or utility target enters the online API.

The existing FeatureStage remains the lifecycle seam. Its original tiny fixtures retain their historical element-count field and are legality tests only. The online extension measures canonical serialized bytes and Python retained-object bytes separately; neither measure is RSS, peak allocation, or a tuple element count. Raw transitions are not retained. Bounded sufficient statistics retain exposure count, reward sum, first and latest source identities only.

Each update requests fixed bounded work from the external work allocator before candidate construction. A prospective copy is measured and submitted for allocation admission before committing learner state. Denied work leaves learner state unchanged; denied allocation leaves learner state unchanged while the already-performed construction work remains charged. Observer decision records may advance. Temporary proposal memory and physical process usage are separate from retained-state accounting.

Seeded deterministic candidate ordering, bounded statistics/features, exact admission traces, birth/revision provenance, and enabled/disabled runs form the acceptance evidence. Creation and revision carry costs; utility remains unavailable. The disabled control leaves the input trajectory and primitive actor unchanged and disables only feature construction. This diagnostic does not claim that the proposed features improve policy quality.

Status: BX-12 local admission, lineage, reconstruction, denial and platform gates passed. Fresh acceptance review, main publication and hosted CI remain pending.

The representation carries a discrete coordinate/value selector and two numeric coefficient statistics (reward sum and exposure count, reported as parameter_count=2). The wire lineage records serialized representation cost separately from retained learner-state memory. Feature lifecycle work is a legacy counter; the external admission ledger records the charged reservation tariff.

Independent reconstruction in evidence/verification/bx-12/replay.py reads the public stream, admission trace and wire lineage without calling OnlineFeatureStage. It reconstructs proposals and state hashes, checks work/storage ledgers and provenance, and rejects changed rewards, missing observations, false work, altered representations, supplied utility, corrupted state and missing lineage. Every live diagnostic invokes this check before succeeding. All four hosted platforms build the real Rust helpers before Python tests.
