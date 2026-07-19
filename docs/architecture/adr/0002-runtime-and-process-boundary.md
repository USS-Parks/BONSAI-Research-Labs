# ADR 0002: Runtime and process boundary

- Status: accepted
- Date: 2026-07-18
- Owner: BONSAI maintainers
- Source: approved PSPR v0.1

## D-02 — Primary runtimes

Decision: Rust stable is the primary runtime for contracts, ingestion, measurement, governance, metrics, CLI, and reporting. Python 3.12 or newer is a separately locked reference-science and adapter package.

Rejected alternatives: Python-only weakens native resource control; Rust-only raises the cost of scientific adapters and comparison reuse.

Consequences: both workspaces have committed locks and independent quality gates; cross-language semantics are tested at versioned contracts.

## D-03 — Adapter process protocol

Decision: evaluated agents and environments run as child processes behind a versioned, length-delimited Protobuf protocol over inherited standard input/output. The observer/governor owns launch and the only telemetry write path.

Rejected alternatives: in-process plugins weaken fault and replay isolation; network RPC adds ports, authentication, firewall, and egress surface.

Consequences: process lifecycle, framing, backpressure, size limits, and derived track classification become contract gates; adapters cannot write canonical telemetry directly.

## D-21 — Containment claim boundary

Decision: hostile native-code sandboxing is excluded from v1. Process isolation, bounded messages, OS resource controls, least-privilege launch, and fail-closed termination protect the instrument from faulty adapters but are not represented as a security sandbox.

Rejected alternatives: claiming hostile-code containment without a dedicated threat model and acceptance milestone was rejected.

Consequences: documentation must state the residual risk; hostile agents cannot be run under a sandbox claim; any revival needs a new threat model, milestone, and explicit authorization.

## Supersession

Only a dated, user-approved PSPR addendum and replacement ADR may change the runtime, trust boundary, transport, or containment claim.
