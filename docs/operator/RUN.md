# Run BONSAI

Reproduce the M1 heartbeat on a clean checkout. This is the supported new-user path.

The workspace does not install `bonsai_reference`. Prefix the heartbeat command with `PYTHONPATH=python/bonsai-reference/src`. Replace `<revision>` with a 40–64 character lowercase hex Git SHA.

```text
PYTHONPATH=python/bonsai-reference/src uv run --frozen python -m bonsai_reference.heartbeat --output target/m1-heartbeat --source-revision <revision> --os-family linux --architecture x86_64
cargo run --offline -p bonsai-report --bin bonsai-report -- target/m1-heartbeat/report-input.json target/m1-heartbeat
cargo xtask bundle-check --root target/m1-heartbeat manifest.json
```

`--os-family` is `windows`, `linux`, or `macos`. `--architecture` is `x86_64` or `arm64`.

The heartbeat is a 32-step diagnostic. It does not run the L profile, does not produce C0–C5 passes, and does not prove physical-host acceptance. See [M1 heartbeat](../experiments/M1-HEARTBEAT.md).

Generated files belong under `target/` or a CI artifact directory. Do not commit them.
