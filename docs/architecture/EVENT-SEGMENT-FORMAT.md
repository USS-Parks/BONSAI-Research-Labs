# Event segment format v1

BC-09 implements the authoritative append-only event segment named by ADR 0003. Segment payloads are opaque encoded event envelopes at this layer; BR-03 remains responsible for validating event meaning before append.

## File names and sequence

A bundle starts at segment sequence zero. Finalized names use `segment-{sequence:020}.bseg`; staging uses the same stem with `.open`. Every later finalized sequence is exactly the previous sequence plus one. The bundle validator rejects duplicates, gaps, and a header sequence that does not match its canonical file name.

## Binary layout

All integers are unsigned little-endian.

| Record | Fields |
|---|---|
| Header | `BNSSEG01`, format epoch `u16`, zero reserved `u16`, sequence `u64`, maximum frame bytes `u32`, zero reserved `u32`, SHA-256 of the preceding header bytes |
| Frame | `BNSFRM01`, encoded length `u32`, exact payload bytes, SHA-256 of the payload bytes |
| Footer | `BNSEND01`, repeated sequence `u64`, frame count `u64`, SHA-256 of the complete header and frame records, SHA-256 of the preceding footer bytes |

The per-segment maximum is immutable in its checksummed header, must be nonzero, and cannot exceed the implementation ceiling of 16 MiB. A writer rejects an oversized frame before writing any part of that frame.

## Finalization and recovery

The writer creates a new `.open` file, appends only complete frame records, writes and synchronizes the footer, atomically publishes the final name without replacing an existing segment, synchronizes the final file, and then removes the staging name. Publication uses a no-clobber hard link; Windows may use its same-volume atomic, no-replace rename when local policy denies hard-link creation.

Recovery does not truncate or rewrite a staged source:

- a complete footer is published as-is;
- a clean end immediately after a complete frame is copied to a new `.recovering` file and finalized there;
- a partial header, frame, checksum, or footer is left untouched and returns its stable error code;
- a stale staged name that is byte-equivalent to the validated final segment is removed;
- a conflicting final segment fails closed.

The committed fixture matrix is [`fixtures/event-segments/v1/expected-outcomes.json`](../../fixtures/event-segments/v1/expected-outcomes.json). It covers valid segments, truncation, frame and segment checksum corruption, oversized frames, duplicate and non-monotonic sequences, recoverable staging files, already-finalized staging files, and unrecoverable partial frames.

## Authority boundary

Segments are immutable raw history. BC-10 may build a replaceable index over them, but neither an index nor later derived tables may mutate or supersede event bytes. In-place compaction is not part of this format.
