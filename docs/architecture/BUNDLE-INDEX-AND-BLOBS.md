# Portable bundle index and content-addressed blobs v1

BC-10 implements the disposable index and immutable blob seams selected by ADR 0003. Event segment files and content-addressed blob files are authoritative. `bundle-index.sqlite3` is never event history and may be deleted and rebuilt from those files.

## Bundle layout

Finalized event segments retain the BC-09 canonical name `segment-{sequence:020}.bseg`. Blob identity is SHA-256 over exact stored bytes. Its only canonical relative path is `blobs/sha256/{first-two-lowercase-hex}/{remaining-62-lowercase-hex}.blob`. The API accepts a typed 32-byte identity; external hexadecimal input must be exactly 64 lowercase hexadecimal characters. Separators, `.` components, uppercase encodings, alternate spellings, symlinks, and paths outside the canonical layout fail closed.

Blob publication writes and synchronizes a staging file before no-clobber publication. A byte-identical existing target is idempotent. An existing target whose bytes do not hash to its path identity is reported as `BLOB_HASH_COLLISION`; caller-supplied expected identity that does not match the new bytes is reported as `BLOB_HASH_MISMATCH`. Neither condition replaces evidence.

## SQLite index and migration

The embedded migration [`0001_bundle_index.sql`](../../crates/bonsai-bundle/migrations/0001_bundle_index.sql) creates a SQLite `STRICT` schema with application ID `1112429385`, user version `1`, and format identity `bonsai.bundle-index/v1`. It contains:

- `event_segments`: canonical path, sequence, frame count, maximum frame size, full-file SHA-256, and byte length;
- `derived_artifacts`: canonical blob path, blob SHA-256, and byte length.

Unsigned 64-bit values use validated canonical decimal text so the index does not narrow BONSAI counters to SQLite's signed integer domain. Rebuild validates every segment through the BC-09 reader, scans only the canonical blob tree, hashes every authoritative file, creates a fresh migrated database, and publishes it as `bundle-index.sqlite3`. No index row is an authority for reconstructing a segment or blob.

`BundleIndex::open_read_only` refuses a missing, symlinked, wrongly identified, unsupported, or path-tampered database. It opens SQLite with `SQLITE_OPEN_READ_ONLY`, enables `PRAGMA query_only`, and exposes query methods only. Windows, Linux, and macOS hosted CI exercise the same bundled SQLite build and read-only fixture.

## Acceptance fixtures

The committed outcome matrix is [`fixtures/bundle-index/v1/expected-outcomes.json`](../../fixtures/bundle-index/v1/expected-outcomes.json). It covers complete rebuild from files, supplied hash mismatch, corrupt/colliding target, traversal-bearing blob identity, traversal-bearing index row, and read-only open. Integration tests also delete the disposable index, rebuild it again, compare every row, and prove that a separately opened SQLite read-only connection rejects mutation.
