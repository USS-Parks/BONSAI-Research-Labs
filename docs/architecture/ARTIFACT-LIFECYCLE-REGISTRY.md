# Artifact lifecycle registry

Status: BR-07 runtime contract
Schema: `bonsai.artifact.v1`
Authority: immutable accepted lifecycle events

## Boundary

`bonsai-lineage` maintains the runtime view of cognitive-artifact identity,
immutable revisions, provenance parents, active consumers, cost history, utility
history, and disposition. It does not decide whether an artifact is useful and
does not infer causal relationships that are absent from accepted events.

The BC-07 contract validator remains the sole transition-legality oracle. The
registry validates the complete candidate prefix before changing state, so an
invalid event cannot partially mutate the registry. Accepted events are retained
in original lifecycle order and can reconstruct the same ordered maps and sets.

## Identity and immutability

- Artifact and revision identifiers are nonzero 16-byte values.
- A birth creates one artifact and its first immutable representation hash.
- A revision names the exact previous revision and creates a new immutable
  version; prior versions remain queryable.
- Parent links name an existing artifact revision and remain provenance edges.
- Consumer link/unlink events change only the active-consumer view; their source
  events remain in the immutable prefix.
- Cost, utility, and disposition histories retain their original messages and
  provenance. The registry does not reinterpret unavailable evidence as zero.
- `replaced`, `retired`, and `removed` are terminal. Later events using the same
  artifact identity fail as terminal resurrection.

## Determinism and failure

The registry uses ordered maps and sets for its public snapshot. Reconstructing
from identical accepted events therefore yields the same artifact order,
revision ownership, versions, active consumers, histories, and terminal state.
The first BC-07 error code is returned for an invalid prefix, and the previously
accepted snapshot and event prefix remain byte-for-byte equivalent at the
Protobuf value level.

The BR-07 model tests cover every registered artifact type; birth, revision,
consumer link/unlink, measured cost, estimated utility, retained,
deprioritized, replaced, retired, and removed transitions; sequence, duplicate,
orphan, unknown-artifact, and terminal-resurrection rejection; and deterministic
direct versus incremental reconstruction.
