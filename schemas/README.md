# BONSAI JSON schema and canonicalization policy

Status: frozen for schema epoch 1 by BC-01.

JSON contracts use the same epoch/minor model as Protobuf. A schema has a stable name, an epoch-bearing URI ending in or containing `/v<epoch>/`, and an explicit `epoch.minor` version. Within an epoch, new properties are optional, existing properties remain present, and property type, requiredness, meaning, and unit do not change. A numeric property always declares its unit.

## Canonical BONSAI JSON bytes

Where a BONSAI contract requires a canonical JSON hash, producers and validators use this transformation before SHA-256:

1. Parse exactly one UTF-8 JSON value. Duplicate object keys are invalid at schema-validation time; non-finite numbers are not JSON and are invalid.
2. Emit no byte-order mark, comments, insignificant whitespace, or trailing newline.
3. Sort object member names in ascending UTF-8 byte order. Preserve array order.
4. Encode strings with standard JSON escaping. String contents are preserved exactly; producers must not depend on platform Unicode normalization.
5. Emit integers in base 10 without a leading plus sign or unnecessary leading zeroes. Other finite JSON numbers use the deterministic shortest representation produced by the repository serializer. Schema-defined units and representations, not a parser's inferred numeric type, carry scientific meaning.
6. Hash the emitted UTF-8 bytes with SHA-256 and render digests as lowercase hexadecimal.

The Rust implementation is exercised by `cargo test` and used by `cargo xtask schema-check` to print the canonical digest of every compatibility fixture. Domain schemas may tighten numeric or string constraints but may not redefine canonicalization.

## Migration obligations

A breaking change requires a new epoch and all of the following in the same governed roster prompt:

- the immutable source-epoch schema and representative source fixture;
- the target-epoch schema and expected migrated fixture;
- a deterministic, version-identified migrator with source and output SHA-256 hashes;
- an explicit account of renamed, defaulted, dropped, or precision-changing data;
- tests for valid migration, invalid input, repeat execution, and interrupted execution;
- retention of the original bytes beside derived migrated bytes, with provenance linking both;
- a declared reader-support window and an exact failure for unsupported future epochs.

A migration never edits a signed or published evidence object in place. If information cannot be migrated without guessing, the result records unavailability or fails explicitly; it does not invent a value.

## Compatibility fixtures

The frozen catalog format under [`../fixtures/schema-compatibility/v1`](../fixtures/schema-compatibility/v1) describes only enough Protobuf and JSON surface to test evolution rules. It deliberately contains no BONSAI domain messages. Run:

```text
cargo xtask schema-check
```

Expected outcomes:

| Fixture | Expected result |
|---|---|
| `additive.json` | compatible |
| `field-renumbering.json` | `FIELD_RENUMBERED` |
| `field-reuse.json` | `FIELD_REUSE` |
| `silent-unit-change.json` | `UNIT_CHANGED` |
| `unversioned-json.json` | `JSON_VERSION_MISSING` |

The Protobuf JSON format itself does not preserve unknown fields, so it is not an unknown-field relay format. See the official [ProtoJSON format guidance](https://protobuf.dev/programming-guides/json/).

## Experiment manifest v1

[`experiment-manifest-v1.json`](./experiment-manifest-v1.json) is the immutable pre-run contract created by BC-03. It is a JSON Schema Draft 2020-12 document with stable URI `https://schemas.bonsai.dev/experiment-manifest/v1` and contract version `1.0`.

Every manifest contains fully resolved values for:

- source repository, revision, and dirty state;
- adapter and environment entrypoints plus configuration objects;
- a non-empty explicit seed set;
- declared track facts, including an explicit replay declaration;
- resource limits, latency deadlines, and energy tier/budget state;
- selected metric versions and parameters;
- scenario identity, version, reward unit, variant, and configuration;
- expected counters, their units, acceptable basis, and run requirement;
- pre-run publication eligibility.

The schema contains no `default` keyword. Omission never delegates a mutable choice to runtime code: a producer must write every required declaration, even when its explicit value is an empty configuration object, `false`, zero replay capacity, or an E0 `null` energy budget. Publication eligibility is not publication authorization.

`cargo xtask schema-check` validates the schema against the Draft 2020-12 meta-schema, rejects any future `default` keyword, validates the canonical fixture, compares LF and CRLF canonical bytes, and exercises the following required-declaration failures:

| Fixture | Expected result |
|---|---|
| `fixtures/experiment-manifest/v1/valid.json` | valid with stable canonical SHA-256 |
| `missing-replay.json` | `MANIFEST_REPLAY_REQUIRED` |
| `missing-resource.json` | `MANIFEST_RESOURCE_REQUIRED` |
| `missing-seeds.json` | `MANIFEST_SEEDS_REQUIRED` |

BC-05 remains responsible for deriving actual track classification from runtime facts. BC-06 remains responsible for the detailed resource-policy and governor-decision contracts. The manifest records their immutable pre-run declarations without claiming either later contract is already implemented.

## Platform inventory v1

[`platform-inventory-v1.json`](./platform-inventory-v1.json) defines the sanitized platform and dependency inventory created by BC-04. The matching Rust collector-boundary types and sanitizer are in `bonsai_contracts::inventory`.

The public inventory retains reproducibility fields for OS version/build/kernel/architecture, CPU and accelerator class, memory size/page size, named clock kind/resolution, drivers, runtimes, compilers, dependency-lock names and SHA-256 hashes, process privilege state, collector status/capabilities/privilege requirements, and thermal/power state. `machine_identity_id` is an opaque UUID assigned independently of hardware identity.

Collector output is untrusted raw input. Before schema decoding, the boundary sanitizer recursively removes forbidden hostname, user-path, source-path, token, API-key, secret, and device-serial fields. It removes their values outright; it does not publish reversible or guessable hashes of secrets or serials. The strict public Rust type and JSON Schema then reject any remaining unknown field.

The frozen fixture pair under `fixtures/platform-inventory/v1/` proves both sides of the boundary: `raw-sensitive.json` is intentionally invalid for public use, while its sanitized output must exactly equal `sanitized-expected.json`, decode through the Rust contract, validate against Draft 2020-12, contain none of the sensitive fixture values, and retain the enumerated reproducibility fields. These are contract and redaction semantics only; BC-04 does not install collectors, probe the live host, establish physical evidence, or claim energy fidelity.

## Track declaration v1

[`track-declaration-v1.json`](./track-declaration-v1.json) records declared intent and the runtime facts used by `bonsai_contracts::track::derive_track`. Declaration never overrides observation. Complete strict single-pass facts derive A; replay or offline update derives B; dense-every-step scheduling derives C; privileged state, human labels, or domain feature targets derive D; observer-data access, incomplete facts, or contradictions derive `INDETERMINATE_TRACK`. The frozen seven-case corpus covers each outcome and declaration mismatch.

## Resource policy v1

[`resource-policy-v1.json`](./resource-policy-v1.json) is the immutable BC-06 external-governor policy contract. Each limit identifies one canonical work class, budget scope, counter, unit, soft limit, hard limit, acceptable measured/estimated basis, and rolling-window duration when applicable. Soft limits may degrade service; hard limits are a distinct violation boundary. The schema has no defaults.

The frozen fixture under `fixtures/resource-policy/v1/` contains all four scopes (`per_event`, `per_step`, `rolling_window`, and `lifetime`), exactly one explicit allocation for each of the nine canonical work classes, and policy-local reason rules covering `admit`, `defer`, `throttle`, `reject`, and `terminate`. Rust semantic validation additionally requires soft limits not to exceed hard limits, complete scope/class/outcome coverage, unique limit and reason identities, every limit to be allocated exactly once to its own work class, and nonzero rolling windows and allocation weights.

The policy's canonical JSON SHA-256 binds the exact JSON bytes to a Protobuf governor decision. The decision contract is documented in [`../proto/README.md`](../proto/README.md). These contracts make a decision reconstructible from recorded evidence; they do not perform measurement, scheduling, or backend enforcement.
