# Analyze a bundle

After [RUN](./RUN.md) produces `target/m1-heartbeat`:

1. Open `target/m1-heartbeat/manifest.json` and confirm every listed SHA-256 matches the file bytes.
2. Read the static report rendered next to `report-input.json`.
3. Re-validate with `cargo xtask bundle-check --root target/m1-heartbeat manifest.json`.
4. Use the local viewer only against a trusted root: see [LOCAL-VIEWER](../reporting/LOCAL-VIEWER.md).

The viewer is local and read-only. It is not a networked dashboard (P-08). Path escape and symlink escapes are rejected.

A valid bundle is not a C0–C5 pass. M1 leaves C0/C1 `not_adjudicated`. Missing counters stay unavailable, never zero.
