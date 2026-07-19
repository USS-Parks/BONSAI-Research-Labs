# Read-only local bundle viewer

BV-03 adds `bonsai-view`, a local command that writes either the canonical `report.html` or one explicitly named bundle file to standard output. This is a file viewer, not a server: it opens no listener and contains no network path. Lineage, metrics, decisions, and comparisons remain ordinary evidence files that can be inspected by relative name.

The viewer canonicalizes the bundle root and every requested existing file. It rejects absolute paths, `.`/`..` components, directories, and any canonical result outside the root, including symlink escapes. Loading the static report also regenerates HTML from `report.json` and requires byte identity with the stored `report.html`. Reads do not alter evidence content or modification timestamps; the viewer exposes no write operation.

```text
cargo run -p bonsai-report --bin bonsai-view -- <bundle-root>
cargo run -p bonsai-report --bin bonsai-view -- <bundle-root> metrics.json
```
