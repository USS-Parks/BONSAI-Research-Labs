# BONSAI Protobuf evolution policy

Status: frozen for schema epoch 1 by BC-01.

This directory will contain BONSAI's portable wire contracts. BC-01 defines how those contracts may evolve; it does not introduce a domain message.

## Version model

- Every contract belongs to an integer **epoch** and a non-decreasing integer **minor** revision. Package and path names carry the epoch as `v<epoch>`; catalog metadata carries both values.
- A minor revision is additive only. It may add a new message, a new enum value, or a new optional field using a never-used field number. Application exhaustiveness still has to be reviewed.
- A breaking semantic or wire change requires a new epoch, a migration record, and retained support for reading the prior frozen fixture. Incrementing the epoch is not permission to destroy the original evidence.
- A schema change and its catalog/fixture update are one atomic commit. An undocumented schema change is invalid even if `protoc` accepts it.

## Field rules

- A field number is permanent once published. Renumbering is deletion plus creation and is rejected within an epoch.
- Deleted fields reserve both their number and name forever. A reserved number or name is never returned to service.
- Field type, presence, cardinality, oneof membership, and unit are schema semantics. They do not change in a minor revision.
- Numeric fields declare a unit. Dimensionless values use `1`; unavailable measurements use an explicit availability contract rather than a sentinel number.
- Fields 19000 through 19999 remain unavailable because Protobuf reserves them for its implementation.
- Consumers must tolerate unknown binary fields. A component that parses and reserializes an envelope must use a supported binary relay path proven to preserve them; ProtoJSON is not that path.

These constraints intentionally tighten the official Protobuf compatibility baseline. The upstream guidance identifies new fields as binary wire-safe, field renumbering as unsafe, and reservation of deleted numbers and names as the safe deletion practice. See the [proto3 updating guidance](https://protobuf.dev/programming-guides/proto3/#updating) and [Proto Best Practices](https://protobuf.dev/best-practices/dos-donts/).

## Compatibility gate

Run from the repository root:

```text
cargo xtask schema-check
```

The command compares the frozen epoch-1 baseline with one additive fixture and four prohibited-change fixtures. It succeeds only when the additive case is compatible and field renumbering, reserved-field reuse, silent unit change, and unversioned JSON are rejected with their named error codes. The fixture catalog is a checker input format, not a BONSAI domain contract.

Fixtures live in [`../fixtures/schema-compatibility/v1`](../fixtures/schema-compatibility/v1). The companion JSON and migration rules are in [`../schemas/README.md`](../schemas/README.md).
