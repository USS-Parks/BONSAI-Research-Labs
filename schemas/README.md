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
