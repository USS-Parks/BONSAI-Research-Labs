# Versioned adapter extensions

BX-09 extends the existing adapter protocol and seven-check AdapterConformanceSuite.
Epoch 1 supports minors 0 and 1. Start offers a range; Handshake selects one supported
version. Every subsequent frame must carry that exact version. A new run is the
only boundary at which capabilities or the selected version can change.

## Required operations and optional capabilities

| Contract | Required behavior | Optional behavior |
| --- | --- | --- |
| Observation | Nonempty declared input types; Step input type must match; payload SHA-256 verified | Additional input types require the caller to understand their payload contract |
| Action | StepResult payload and SHA-256 | Scientific action semantics belong to the adapter/environment integration |
| Work | Explicit work flag, including false | Work/WorkResult only when work is true |
| Feedback | Explicit feedback flag, including false | Feedback/ack only when feedback is true |
| Artifact events | Explicit asynchronous-events flag; typed event declarations | ArtifactLifecycleEvent requires asynchronous events and its declared type; downstream envelope/lineage validation still applies |
| Checkpoint | No checkpoint restore operation is supported in this adapter epoch | A required checkpoint capability is refused; BX-08 observer recovery does not restore agent state |
| Replay and access | All existing replay, offline, observer, privilege, filesystem and network flags remain explicit | A capability declaration never replaces resource or isolation authorization |

Minor 1 adds repeated required_capabilities and optional_capabilities fields.
Minor 0 must leave both empty. Supported requirement names are
bonsai.observation/v1, bonsai.action/v1, bonsai.work/v1, bonsai.feedback/v1,
and bonsai.artifact-events/v1. Work/feedback/artifact requirements also require
their corresponding existing flags. Artifact events additionally declare
bonsai.artifact.v1.ArtifactLifecycleEvent. Unknown requirements, including
bonsai.checkpoint/v1, fail with ADAPTER_PROTOCOL_REQUIRED_CAPABILITY_UNSUPPORTED.

Optional names are hints only. Unknown hints are ignored and confer no authority.
Both lists together have at most 32 unique names, each 1–128 ASCII letters, digits,
dots, slashes, underscores or hyphens. Duplicates, overlap, and invalid names fail.
The existing capability fingerprint includes both lists and remains immutable.

## Wire compatibility and refusal

Protobuf endpoints skip unknown optional envelope fields. Retained evidence must
keep original raw bytes when byte preservation matters; decoding and re-encoding
with Prost is not a transparent relay. An unknown field inside a capability
declaration cannot silently alter its fingerprint: a hash containing unknown bytes
does not equal the known decoded declaration's fingerprint and is refused.
Use the explicit optional-name list for ignorable capability hints.

An unsupported epoch or minor is refused during Handshake even if a broad Start
offer included it. Epoch changes require an explicit migration; this release
provides tested refusal, not an invented migration. Existing schema compatibility
fixtures still reject field renumbering, field reuse, unit changes and unversioned
JSON; the breaking-epoch fixture exercises EPOCH_CHANGE_REQUIRES_MIGRATION.

## Historical verification and evidence

The preserved v1.0 generated bindings and proto are byte copies from main commit
86061d89a1b3505a8a4a7f77ad679192cafb0b8a. The v1.1 fixture uses newly generated
bindings from the locked compiler. The same generic runtime launches both fixtures,
records real exchanges, lifecycle journals and launch audits, checks repeated action
digests, and contains an adapter waiting for input after a completed handshake.
Both feed the same seven-check certification suite. These fixtures prove adapter
contract compatibility; they do not certify scientific quality or hostile-host isolation.

The governed verifier accepts either its complete current supported reference-source
set or the complete preserved historical set. It does not accept arbitrary mixtures
or trust source hashes supplied by a bundle. Existing archived governed bundles
remain independently verifiable. The scientific verifier retains its explicit
reference-program scope; generic runtime, governor and platform authority do not
import that scientific implementation.

Run the live probe with an absolute virtual-environment Python executable and a
new output path:

```text
cargo run -p bonsai-runtime --example adapter_compatibility -- PYTHON NEW_OUTPUT
```

The probe retains two trials for each of seeds 7, 42 and 91 at each minor version,
plus real timeout evidence. Formal Windows/Linux results, source identities,
compatibility matrix and dependency audit are retained under evidence/verification/bx-09.
