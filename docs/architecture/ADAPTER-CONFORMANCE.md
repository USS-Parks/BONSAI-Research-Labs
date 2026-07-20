# Adapter runtime conformance suite v1

Status: BR-10 adapter-certification authority for protocol epoch 1.

The `AdapterConformanceSuite` converts observer-produced runtime evidence into one deterministic machine report. A certification says that the submitted adapter run conformed to BONSAI's runtime contracts. It does not certify learning performance, scientific quality, safety against hostile native code, or eligibility for a claim-ladder pass.

## Required checks

Every report evaluates these dimensions exactly once and in this order:

1. `protocol` replays the peer-attributed frame transcript through the BR-01 state machine and requires a valid stopped terminal state.
2. `isolation` checks the BR-06 capability audit for the three granted environment keys, the three bounded protocol/diagnostic handles, a declared working directory, and no observer-path exposure.
3. `ordering` reuses the BR-04 partial-order classifier and rejects duplicate IDs, missing parents, source-sequence conflicts or gaps, cycles, clock regression, or invalid bounded input. Late arrival alone remains an annotation rather than a failure.
4. `lifecycle` validates zero-based ordinals, every BR-05 state transition, and a completed or recovered terminal state.
5. `determinism` requires at least two valid lowercase SHA-256 probe results and exact equality.
6. `timeout` requires an intentionally exercised read or shutdown deadline, the corresponding stable transport code, process containment, and an observed duration at or beyond the declared deadline.
7. `classification` derives the BC-05 track from runtime facts and requires it to match both the declaration and the certification's expected track. Indeterminate facts remain indeterminate.

A failed check makes the adapter `rejected`. With no failures, any missing or indeterminate check makes the report `indeterminate`. Only seven passing checks produce `certified`. Invalid adapter IDs produce no report.

## Reference corpus and portability

`fixtures/adapter-conformance/v1/expected-outcomes.json` freezes a good adapter plus independently bad protocol, isolation, ordering, lifecycle, determinism, timeout, hidden-replay, observer-access, and missing-evidence cases. The integration harness constructs the isolation audit through the real launch policy and applies real protocol frames to the production state machine. Existing process-transport tests provide the live stalled-child containment primitive used by the timeout evidence contract.

The same committed corpus runs in the required Windows x86_64, Linux x86_64, macOS arm64, and macOS Intel hosted jobs. Equality is semantic machine-report equality; the certification contains no host path, timing performance, or scientific score.

## Third-party use and trust boundary

A third-party runner supplies a peer-attributed transcript, launch-policy audit, observed events, lifecycle journal, repeated output hashes, timeout-probe result, and track declaration to the suite. Those inputs must be collected by the observer or another trusted certification runner; an adapter's unauthenticated self-report is not evidence. Later integrity work may bind the report into a signed bundle, but BR-10 does not make a signing or hostile-code sandbox claim.
