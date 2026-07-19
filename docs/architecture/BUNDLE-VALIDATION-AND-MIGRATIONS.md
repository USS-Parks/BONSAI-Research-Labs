# Bundle validation and migration contract v1

BC-12 provides one whole-bundle conformance report without turning validation into scientific claim adjudication. The `cargo xtask bundle-check [--root PATH] MANIFEST` command writes exactly one JSON report to standard output. `VALID`, `MIGRATABLE`, `FORWARD_READABLE`, and `VALID_WITH_LIMITATIONS` return exit code 0; `INVALID` and `INDETERMINATE` return exit code 2. An operational failure that prevents a report, such as an unreadable manifest or unsafe root, returns exit code 1 with a stable error code on standard error.

## Validation dimensions

The report schema is `bonsai.bundle-validation-report/v1`. It always records the manifest epoch, access posture, overall verdict, stable top-level reason codes, and eight independent checks:

| Check | Contract |
|---|---|
| `schema` | Current manifest and typed component JSON conform to their frozen Draft 2020-12 schemas. |
| `hashes` | Every required regular file exists beneath the trusted root, traverses no symlink, and matches its lowercase SHA-256 identity. |
| `track` | The declared track agrees with deterministic derivation; incomplete or boundary-violating facts remain indeterminate. |
| `inventory` | Platform inventory is structurally valid and unavailable collectors remain explicit limitations. |
| `resource_policy` | The referenced resource-policy document conforms to the versioned policy schema. |
| `failures` | The failure log is readable and any fatal record makes the bundle invalid. |
| `metric_provenance` | Metric inputs bind hashes of actual bundle content, not merely unverified manifest declarations. |
| `migrations` | The epoch is current, deterministically migratable, or a future epoch restricted to read-only access. |

`VALID_WITH_LIMITATIONS` preserves a structurally valid bundle whose declared counter evidence is unavailable. `INDETERMINATE` is reserved for track facts that cannot establish an eligible track. Neither verdict estimates scientific quality, applies a claim criterion, or converts missing observations to zero.

## Epoch behavior

Epoch 1 is validated directly. The supported epoch-0 manifest migrates to epoch 1 in memory through the named `bonsai.bundle-migrations/v1` registry. Migration is deterministic, reports the SHA-256 of the migrated bytes, and never rewrites the source bundle. Unsupported older epochs fail explicitly.

A future epoch is never decoded as if it were current. The validator reads only its stable header and file identities, verifies required-file hashes, marks current semantic checks `not_run`, and returns `FORWARD_READABLE` with `read_only` access. A future bundle with a missing or mismatched required file is invalid rather than forward-readable.

The committed [seven-case corpus](../../fixtures/bundle-validation/expected-outcomes.json) freezes exact outcomes for current valid, old migratable, forward readable, component-corrupt, ambiguous-track, unavailable-counter, and tampered bundles. The [manifest schema](../../schemas/bundle-manifest-v1.json) and [report schema](../../schemas/bundle-validation-report-v1.json) are included in the repository-wide schema gate.
