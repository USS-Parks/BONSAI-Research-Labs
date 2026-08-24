# BRDC-1 feature and subproblem stages

BE-05 implements Track A feature candidates and reward-respecting feature-attainment subproblems. Features are integer detectors with immutable lineage, bytes, work, and consumers. Producing a feature records cost and leaves utility unset. Semantic labels are rejected and never used as learning input.

A Track A subproblem is reward-respecting: success requires attainment and a positive original reward. Reward-oblivious posing is refused on this track.

The committed fixture at `fixtures/brdc1-features/v1/expected-outcomes.json` reconstructs birth, revision, retirement, consumer links, and exact work.
