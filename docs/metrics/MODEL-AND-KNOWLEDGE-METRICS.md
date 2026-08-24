# Model and knowledge metrics

BK-08 measures one-step and option-horizon prediction error, reward/stopping calibration, jump-length error, drift/recovery, uncertainty, harmful planning, and semantic stability. Errors are compared only when models share a prediction target. A representation change with the same target is classified as representation-shift; a different target yields `MODEL_TARGET_UNALIGNED` rather than a numeric delta.

The committed fixture at `fixtures/model-metrics/v1/expected-outcomes.json` classifies stale, biased, calibrated, and representation-shift models.
