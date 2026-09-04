# BONSAI threat model v1

Status: BV-11 authority  
Owner: security lead  
Machine table: [`fixtures/threat-model/v1/boundaries.json`](../../fixtures/threat-model/v1/boundaries.json)

> **D-21 / P-05 — not a hostile-native-code sandbox.**  
> BONSAI v1 is designed to tolerate **faulty** adapters through process isolation, bounded messages, OS resource controls, least-privilege launch, and fail-closed termination. Those controls are **not** a security sandbox and **must not** be represented as containment for hostile arbitrary native code. If hostile-code evaluation is required, a new threat model, containment architecture, and explicit authority must precede it.

This document maps every v1 trust boundary to controls, tests, residual risk, and prompt IDs. It does not authorize privileged collectors, publication, or a sandbox claim.

## Trust boundaries

| ID | Boundary | Controls | Tests | Residual risk | Prompts |
|---|---|---|---|---|---|
| TB-01 | Faulty or malformed agents | Length-delimited stdio protocol; fail-closed decode; least-privilege launch; fail-closed termination | `crates/bonsai-runtime/tests/process_transport.rs`; `crates/bonsai-runtime/tests/adapter_conformance.rs`; `crates/bonsai-accept/tests/m4_acceptance.rs` | Native code can use ambient OS APIs outside BONSAI-controlled handles. This is not a sandbox. | BR-02, BR-06, BQ-07, D-21, BV-11, BV-12 |
| TB-02 | Event floods | Source authorization; envelope and payload caps; fixed-window rate limits; bounded rejection ledger | `crates/bonsai-ingest/tests/ingest_validation.rs`; `crates/bonsai-accept/tests/m4_acceptance.rs` | A privileged collector or host process outside ingest can still consume host resources. | BR-03, BC-09, R-08, R-09, BV-12 |
| TB-03 | Artifact bombs | Hard byte and file caps; graph-depth bound; create-new hashed blobs; fail-closed overflow | `crates/bonsai-governor/tests/agent_storage.rs`; `crates/bonsai-accept/tests/m4_acceptance.rs` | Caps apply to BONSAI-brokered paths. Unbrokered ambient writes are outside the claim. | BQ-06, BR-07, BV-12 |
| TB-04 | Path and symlink attacks | No-follow metadata; work-root resolution; denied observer paths; hash-checked input copies | `crates/bonsai-runtime/tests/agent_isolation.rs`; `crates/bonsai-governor/tests/agent_storage.rs`; `crates/bonsai-accept/tests/m4_acceptance.rs` | A process with ambient filesystem APIs can still leave the declared tree. D-21 applies. | BR-06, BQ-06, BV-12 |
| TB-05 | Bundle tampering | SHA-256 file identities; symlink-refusing validator; optional HMAC-SHA256 envelopes; independent off-host verify | `crates/bonsai-bundle/tests/bundle_validation_corpus.rs`; `crates/bonsai-accept/tests/m4_acceptance.rs` | HMAC uses an operator-supplied local key. There is no external key service. | BC-12, BV-12, BV-16 |
| TB-06 | Observer-to-agent leakage | Cleared environment; copied inputs only; protocol observer-path denial; `INDETERMINATE_TRACK` on observer access | `crates/bonsai-runtime/tests/agent_isolation.rs`; `crates/bonsai-runtime/tests/storage_guard.rs` | Isolation is an interface contract, not an adversarial OS sandbox. | BR-06, BR-09, BQ-06, R-07, BV-11 |
| TB-07 | Secrets | Inventory field removal; `cargo xtask verify --redact`; fixture inventory sanitizer; no raw environment dumps | `crates/bonsai-xtask/src/main.rs`; `crates/bonsai-xtask/src/schema_check.rs`; `crates/bonsai-accept/tests/m4_acceptance.rs` | Redaction is defense in depth. Operators must not pass secrets on command lines. There is no in-tree publication secret-scan job. | BC-04, BG-04, BG-06, BV-12, BV-16 |
| TB-08 | Local dashboard | Local-only viewer; rejected absolute/parent/symlink paths; no listener or SaaS dashboard in v1 | `crates/bonsai-report/src/viewer.rs`; `crates/bonsai-report/tests/m3_reporting.rs` | A local HTML file can still be opened by a browser with the operator's privileges. P-08 parks multi-user services. | BV-03, BV-08, P-08, BV-11 |
| TB-09 | Dependency and build risks | Committed `Cargo.lock` and `uv.lock`; [dependency policy](../governance/DEPENDENCY-POLICY.md); SBOM generation; offline rebuild scripts; explicit waivers only | `crates/bonsai-accept/tests/m4_acceptance.rs`; `scripts/generate_sbom.py`; `scripts/offline_restore.py` | A clean physical machine restore is separately attested. CI offline tests are not that attestation. | BG-05, D-08, BV-13, R-15 |
| TB-10 | Privileged collectors | Opt-in collectors; capability matrices; fail-closed unsupported hard controls; explicit authority for privileged use | `crates/bonsai-platform/tests/os_backends.rs`; `crates/bonsai-governor/tests/m3_governor.rs` | When a collector is authorized and privileged, host integrity risk increases. E3/lab probes stay parked (P-01). | BM-13, BQ-07, R-14, BV-11, BV-12 |

## Related contracts

- [Security policy](../../SECURITY.md)
- [Agent/observer isolation](../architecture/AGENT-OBSERVER-ISOLATION.md)
- [Event ingestion validation](../architecture/EVENT-INGESTION-VALIDATION.md)
- [Bundle validation](../architecture/BUNDLE-VALIDATION-AND-MIGRATIONS.md)
- [Run lifecycle and recovery](../architecture/RUN-LIFECYCLE-AND-RECOVERY.md)
- [Local viewer](../reporting/LOCAL-VIEWER.md)
- [Publication policy](../governance/PUBLICATION-POLICY.md)
- [Parked scope](../governance/PARKED-SCOPE-LEDGER.md)

BV-12 implements the hardening suite against this model. BV-13 owns supply-chain restore. Hostile sandbox work remains parked.
