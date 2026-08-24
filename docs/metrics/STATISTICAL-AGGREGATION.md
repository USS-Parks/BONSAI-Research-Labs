# Statistical aggregation and multiplicity

BK-13 aggregates preregistered paired-seed outcomes. It reports paired mean-difference effect sizes, deterministic bootstrap percentile intervals, missing and failed runs, jackknife sensitivity, and Holm-adjusted rejections for a declared comparison family.

Post-hoc seed removal and undeclared metrics fail closed. A pair present in the data but absent from both the declared schedule and the declared exclusion list is `STAT_UNDECLARED_EXCLUSION`. An outcome absent from the preregistered list is `STAT_UNDECLARED_METRIC`.

The committed fixture at `fixtures/statistical-aggregation/v1/expected-outcomes.json` is produced by the independent Python reference in `python/bonsai-reference/src/bonsai_reference/stats.py` and must match the Rust engine bit-for-bit.
