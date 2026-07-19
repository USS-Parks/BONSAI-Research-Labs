# Adapter protocol v1

Status: BR-01 protocol authority for schema epoch 1.

BONSAI supervises evaluated agents as child processes. The adapter protocol is a versioned, ordered Protobuf state machine carried over inherited standard input and standard output; BR-02 owns the bounded length framing and process implementation. No network transport or learning-algorithm API is defined here.

## Roles and negotiation

The supervisor sends `Start` with the run identity, accepted version interval, deterministic run seed, and a monotonic deadline. The adapter returns exactly one `Handshake`, selecting a compatible version and declaring all runtime capabilities. The declaration is encoded in protobuf field order and SHA-256 hashed; the adapter includes that fingerprint on every subsequent adapter-originated frame, and `Configure` binds the exact accepted fingerprint. A changed fingerprint, second handshake, or use of an undeclared operation fails closed.

Capabilities cover reset, external work, authorized feedback, asynchronous events, accepted input and emitted event types, transition retention, offline updates, observer-data access, privileged-state access, filesystem read/write, and network access. These are runtime facts used by the BC-05 track classifier; declaration does not grant a capability or make Track A eligible.

Protocol epoch 1 minor 0 is the only current version. Unknown additive fields remain subject to the repository Protobuf evolution policy. Epoch mismatch and a selected minor outside the offered interval are terminal protocol violations rather than implicit downgrade.

## State machine

```text
created --Start--> awaiting-handshake --Handshake--> awaiting-configure
  --Configure--> awaiting-configure-ack --Ack--> ready
  --Reset--> awaiting-reset-ack --Ack--> active
  --Step--> awaiting-step-result --StepResult--> active
  --Work--> awaiting-work-result --WorkResult--> active
  --Feedback--> awaiting-feedback-ack --Ack--> active
  --Stop--> awaiting-stopped --Stopped--> stopped
```

`Reset` may begin from ready or active. `Event` is accepted only while active and only when asynchronous events were declared. `Stop` may be issued from any live state so a failed exchange can terminate cleanly. Every message after `Stopped`, including another `Stop`, is rejected as post-stop evidence.

Supervisor and adapter sequences are independent, start at zero, and advance only after a valid frame. An invalid frame never advances the state or sequence. The supervisor is the only sender of start, configure, reset, step, work, feedback, and stop. The adapter is the only sender of handshake, results, acknowledgements, events, stopped, and protocol errors.

## Inputs, hashes, seeds, and deadlines

Run and episode identities and work/feedback identities are nonzero 16-byte UUID representations. Configuration, input, action, work, result, and feedback bytes are bound to exact SHA-256 values where the message carries a digest. Input types must have been declared in the handshake.

The supervisor supplies the run seed in `Start` and the episode seed in each `Reset`; zero is a valid deterministic seed. Adapters must not replace either seed with ambient randomness without declaring the resulting behavior outside deterministic conformance.

Deadlines are absolute values in the supervisor's monotonic clock domain. They are strictly increasing within a session. The adapter treats them as response bounds, not cross-process timestamps, and does not compare its local clock value directly to them. BR-02 turns expiry into bounded transport/failure evidence.

## Explicit exclusions and failure posture

The protocol grants neither observer files nor replay history. `Feedback` is a manifest-authorized online signal, not a route for retained observer telemetry. Retained transitions, offline updates, observer-data access, and privileged inputs are explicit track-classification facts and are never inferred false from omission.

This protocol is not a hostile-native-code sandbox. Process containment, filesystem launch policy, and observer isolation are implemented and tested by BR-02 and BR-06. Invalid ordering, sender, sequence, version, capability fingerprint, capability use, identity, digest, deadline, or post-stop traffic yields a stable bounded rejection and no state advance.

The committed BR-01 outcome matrix is `fixtures/adapter-protocol/v1/expected-outcomes.json`. Rust conformance tests cover the valid lifecycle and exact rejection classes for configure-before-start, incompatible version, changed capability fingerprint, and post-stop traffic.
