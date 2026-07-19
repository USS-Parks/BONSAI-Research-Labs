# M1 auditable heartbeat

BE-03 runs the certified primitive, batch-one, no-replay controller for 32 steps in `diagnostic.stable-bandit`. The external semantic-work budget admits 128 work items against a hard limit of 160 and reports 32 items of headroom. The frozen result is 30 cumulative reward, with action, reward, update, parameter-touch, and work counts included in the machine summary.

Each CI matrix job emits a fresh schema-valid `bonsai.bundle/v1` directory, platform inventory, static machine/HTML report, lineage, metrics, decisions, comparisons, and the frozen semantic summary. The bundle records its Windows, Linux, macOS arm64, or macOS Intel row separately. An aggregate hosted job downloads all four bundles and requires one exact semantic hash plus the complete platform-row set.

The overhead row is a deterministic semantic-fixture value, not physical timing acceptance. Energy remains E0 and thermal state unavailable. C0 and C1 inputs are reportable and rule versioned, but their verdict is `not_adjudicated` because concrete C0/C1 adjudication remains BV-04 outside M1. No C0–C5 pass, physical-host acceptance, or instrument-completion claim follows from this heartbeat.

Local reproduction uses a clean 40–64 character source revision:

```text
uv run --frozen python -m bonsai_reference.heartbeat --output target/m1-heartbeat --source-revision <revision> --os-family <family> --architecture <arch>
cargo run --offline -p bonsai-report --bin bonsai-report -- target/m1-heartbeat/report-input.json target/m1-heartbeat
cargo xtask bundle-check --root target/m1-heartbeat manifest.json
```

Set `PYTHONPATH=python/bonsai-reference/src` for the module command when the package is not installed. Generated output belongs under `target/` or the CI artifact directory and is not committed; the stable expected summary is committed under `fixtures/m1-heartbeat/`.
