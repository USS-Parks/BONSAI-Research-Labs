# Governed primitive experiment runner

BX-05 composes the existing runtime, delegated Linux authority, governor, ingestor, bundle writer, primitive learner, causal environment, and report generator. Linux is required. WSL2 results are virtualized evidence; physical-host acceptance and C0/C1 adjudication remain separate gates.

## Operator command

Run from this checkout inside an owned delegated cgroup v2 unit. The empty delegated parent must grant cpu, memory, and pids controllers. The runner creates and retires only its two UUID-named child groups.

```sh
cargo xtask run --manifest MANIFEST.json --output NEW_RUN_DIRECTORY --authority AUTHORITY
```

Optional `--cancel-file PATH` requests cancellation when that file appears. The output directory must be new. The immutable manifest specifies component paths/configuration, one seed, source revision and dirty-patch identity, counters, metrics, and resource limits. The environment receives scenario configuration; the primitive agent receives its action count, public observations, and rewards. Launch-path isolation is not arbitrary hostile-code filesystem isolation.

S fixes 2,000 steps, 120 seconds, 1 GiB agent RSS, 64 MiB agent storage, 512 MiB observer output, 10 ms per-step agent CPU, and 50 ms dispatch-to-validated-action-response time. This cut supports E0 and declared Track A intent. Custom profiles cannot exceed those bounds. Episode resets derive seeds from the base seed while preserving learned parameters.

The resolved policy is written before launch. Each child has a one-CPU quota and 32-task ceiling; the environment has 128 MiB charged memory. Actual controller readbacks and hierarchical measurements are retained. Acting and learning work are separately admitted and charged, including a rolling acting-work allowance. Other work classes are never dispatched. The v1 contract requires all classes and outcomes; equal soft/hard limits make soft-only decisions unreachable. These declarations do not replace BX-06 adjudication.

## Completion and evidence

`observer/run-status.json` is authoritative. It starts INCOMPLETE and becomes COMPLETE only after execution and finalization checks pass. Execution-only telemetry/report status cannot override it. Failure paths retain classified incomplete results and partial measured rewards; absent observations remain unavailable.

The observer reserves 128 KiB for metadata and closeout, including 8 KiB for control records. Metadata writes reject over-allocation before writing, and event frames have a separate exact byte allowance. Large manifests can therefore be rejected even with unused event capacity. Final accounting includes all observer files. The existing broker inspects agent storage. Cancellation and wall expiry terminate the owned groups during pending operations; cleanup and failure reporting follow. Reports are generated after action exchange, outside the action deadline.

`observer/bundle-manifest.json` contains scientific v1 roles. `observer/artifact-index.json` additionally binds identities, uncertainty, resolved controls, reports, and lifecycle; final status hashes that index. The index excludes itself and mutable completion markers explicitly. Source identity includes untracked implementation files and the operator binary hash.

## Reproduce acceptance diagnostics

With the existing locked Rust cache and Linux virtual environment prepared:

```sh
sh evidence/verification/bx-05/failure-matrix.sh s_profile
sh evidence/verification/bx-05/reconcile.sh target/PRINTED_BATCH/s_profile
sh evidence/verification/bx-05/failure-matrix.sh exit memory storage delay cancel cancel_pending wall observer finalization
```

The helper creates a delegated systemd user unit, uses new UUIDs/output roots, and prints the retained batch directory. It neither elevates nor changes global controllers. The wall diagnostic deliberately expires before launch; pending-operation cancellation is a separate live case. Independent reconciliation checks segment/payload hashes, event continuity, required populations, resource bounds, reward totals, cleanup, and indexed artifacts. It does not adjudicate C0/C1.

```sh
cargo xtask bundle-check --root RUN_DIRECTORY/observer bundle-manifest.json
```

The bundle manifest argument is relative to the supplied root. Track facts remain incomplete pending BX-06, so a complete execution can correctly remain INDETERMINATE for publication. Failure bundles retain their fatal failure.
