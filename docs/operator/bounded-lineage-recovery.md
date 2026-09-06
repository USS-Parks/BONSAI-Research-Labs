# Bounded lineage evidence and interrupted-run recovery

BX-08 adds durable storage at the existing contract and segment seams. Full replay
remains the conformance oracle. The in-memory registry remains available for
bounded callers; long histories use `PersistentLineageRegistry`.

## Admission and commit

`create` requires a new owned output directory and explicit limits for live
artifacts, consumers per artifact, encoded frame size, database size and total
output. Historical revisions, cost/utility identities, retired artifacts and
ancestry stay in SQLite. The shared contract engine receives only the current
artifact and referenced identities. The database cache is 2 MiB, mmap is disabled,
and ancestry traversal uses indexed disk rows rather than a growing memory queue.

`append` validates a provisional event. `commit` synchronizes and publishes its
segment before atomically committing the segment reference, event count and
validated state. The index commit is the acceptance boundary. A failure after that
boundary can return an error while the batch is already durable; reopening reports
the exact accepted prefix. A failed storage session is poisoned and must be reopened.

The store takes an exclusive SQLite writer lock. Contract rejections leave the
candidate state unchanged. Live-artifact and consumer limits reject admission;
database and output exhaustion stop the session. Five database allowances plus
64 KiB are reserved for the index, journals, bounded recovery audit and metadata.
Retained extra recovery files also count against the output allowance.

## Recovery and historical queries

`open` validates every committed segment by streaming frames, checks checkpoint
identity and row hashes, and rebuilds a disposable bounded-cache index from the
durable events. It compares all reconstructed artifact, ownership, history and
ancestry state with the checkpoint. Unsupported schemas and corrupted state fail.
Unindexed attempts remain on disk as orphan evidence and are never silently admitted.

Every reopen is a continuation. It does not restore an agent, its random state,
learning state or an uninterrupted Track A execution. Historical revision ownership
and ancestry remain queryable after artifact retirement. Arbitrary ancestry queries
can visit many historical relations and consume their declared disk allowance;
there is no constant-time ancestry claim.

## Recover a governed observer prefix

```text
cargo xtask recover-run --root <OBSERVER_PATH> --output <NEW_RECOVERY_PATH> --maximum-output-bytes <N>
```

The portable command supports the governed runner's current single-segment layout.
It captures the original segment in the existing content-addressed blob store with
a bounded streaming read, retains the original file, and copies checksum-verified
complete frames to a new segment. A truncated tail is explicitly recorded; checksum
corruption fails. It verifies the governed source sequence, causal predecessor chain
and a consistent run identity within the prefix, then rebuilds the disposable index.

The resulting `recovery.json` always says `INCOMPLETE`, `continuation: true`,
`uninterrupted_track_a_eligible: false` and `agent_state_restored: false`.
This derivative cannot pass `verify-run` as a completed governed execution.
Recovery requires enough declared space for source preservation, segment publication
and index metadata; insufficient quota fails before creating the output root.

## Derived data and blobs

`materialize_derivation_stream` accepts bounded typed row batches and a committed
expected row count. It enforces batch rows/bytes, rows per file and output bytes,
flushes each batch, and preserves the existing semantic hash across batch boundaries.
Rotate files at the declared file-row cap to bound Parquet footer metadata.
`validate_derivation` now reads batches incrementally instead of collecting them.
`put_blob_stream` uses a 64 KiB buffer and explicit byte cap.

## Verification boundary

The generated differential corpus covers 2,880 admissions: 1,810 accepted and 1,070
rejected, with matching contract errors and recovered state. Portable tests cover
all four segment/index commit boundaries, corruption, SQLite exhaustion, writer
exclusion, retired ancestry, quota preservation and checkpoint compatibility.
Derived-table fixtures preserve all four table semantics and exercise output caps.

The final live harness is `evidence/verification/bx-08/run.py`: three fresh-process
trials at 1k, 10k and 100k events, each followed by fresh recovery, plus abrupt process
exits at all four commit boundaries. The live-artifact cap stays one and batches
hold at most 1,000 events. RSS values are current process-tree snapshots, not peak.
The gate limits median RSS spread to 16 MiB and per-process growth to 32 MiB.

The separate disk-full harness uses a private 2 MiB tmpfs in a WSL2 user/mount
namespace, preserves the failed run, and independently checks the recovered prefix.
The governed interruption harness kills the actual operator process group after
durable telemetry appears, verifies empty owned descendant groups, and reconciles
the recovered prefix against raw checksummed frames.

Exact final records, timings, archive hashes and hosted publication status are
recorded in the DEVLOG and verification ledger. WSL2 evidence is explicitly
virtualized Linux and does not close physical-host acceptance.
