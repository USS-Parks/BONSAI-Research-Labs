# Lineage checkpoint v1 compatibility fixture

The v1 directory is copied from the final native BX-08 process-exit corpus, after an abrupt exit immediately following the second index commit. It contains two accepted events in two committed batches. A subsequent open is explicitly a continuation.

Source batch: `target/bx08-live-1788668992536634500/crash-after_index_commit`. The exact source hashes and executable identity are retained in the BX-08 live evidence archive and verification records. Tests copy the fixture before opening it, so its committed bytes remain unchanged.
