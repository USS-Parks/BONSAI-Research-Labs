# Verification record v1

`cargo xtask verify` runs one command without a shell and appends one JSON object to `<record-dir>/records.jsonl`. Its schema identifier is `bonsai.verification-record/v1`.

## Usage

```text
cargo xtask verify --prompt BG-06 --record-dir evidence/verification --evidence-class local --runner-class unknown -- cargo test --workspace --all-features
```

Repeat `--redact <literal>` for sensitive text that must be replaced with `<redacted>` in recorded command arguments and captured output. Do not pass secrets on command lines; redaction is defense in depth, not authorization to handle credentials.

## Required fields

- record and prompt identifiers;
- redacted command vector and sanitized `<repository-root>` working directory;
- source revision and pre-command dirty state;
- OS, architecture, evidence class, and runner class;
- UTC Unix-nanosecond start/end plus monotonic duration;
- child exit code and pass/fail result;
- relative stdout/stderr artifact paths, byte counts, and SHA-256 hashes.

The task runner does not enumerate or serialize the environment. It is not a hostile-command sandbox and does not yet bound output volume; later security prompts own those controls. Callers retain only appropriately scoped, redacted artifacts.
