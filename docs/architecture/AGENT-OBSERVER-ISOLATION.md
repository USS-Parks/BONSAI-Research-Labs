# Agent and observer data isolation v1

Status: BR-06 launch-policy and capability-audit authority for schema epoch 1.

BONSAI separates the adapter-facing agent tree from observer telemetry, index, and report trees. The launch policy grants only copied manifest-authorized inputs, an agent-owned writable directory, inherited stdin/stdout protocol pipes, and independently bounded stderr. Observer paths are absent from process arguments, environment values, working directory, and outbound protocol payloads.

## Filesystem layout and grants

The supervisor creates `agent/inputs`, `agent/work`, `observer/telemetry`, `observer/index`, and `observer/reports` beneath its run root. An authorized input must have a safe logical name, a lowercase manifest-bound SHA-256, be a regular non-symlink file, and be no larger than 64 MiB. It is copied with create-new semantics into `agent/inputs`; the copied bytes must match the manifest hash and the copy is marked read-only before the grant is returned, while a mismatch removes the rejected copy. The grant records the copied byte count and SHA-256 but never exposes the source path. Duplicate, traversal-shaped, directory, symlink, hash-mismatched, stale, and out-of-root grants fail closed.

The adapter starts in `agent/`, receives each copied input path through an explicit `--bonsai-input` argument, and receives `agent/work` as its only declared writable location. This is a supervisor interface contract: later BQ-06 owns metering and enforcement of persistence, path traversal, and symlink behavior.

## Launch and protocol surface

The process environment is cleared before the policy adds exactly `BONSAI_AGENT_ROOT`, `BONSAI_INPUT_ROOT`, and `BONSAI_WORK_ROOT`. The configured inherited handles are stdin protocol, stdout protocol, and bounded diagnostic stderr. The capability audit records the working directory, arguments, environment keys, and those three handles and fails if an observer-tree path appears.

Outbound protocol bytes are checked for the canonical observer-tree paths before transport mutation. An ordinary manifest-authorized online payload passes; a telemetry, index, or report path fails with `ISOLATION_OBSERVER_PATH_EXPOSURE`.

## Capability request and track consequence

A deliberate request for observer telemetry, index, or report access is denied with `OBSERVER_ACCESS_DENIED`. The request is also recorded as the BC-05 `observer_data_access` runtime fact. BC-05 then derives `INDETERMINATE_TRACK` with `OBSERVER_DATA_BOUNDARY_VIOLATION`; a Track A declaration cannot override it.

## Verification and explicit limit

The real Python inspection adapter enumerates only its granted arguments, environment, protocol handles, current directory, copied input tree, and writable tree. It reads the authorized input, writes its work probe, and cannot discover the observer canary through those granted interfaces. Fixtures also prove observer paths are rejected from arguments and protocol payloads and that an observer-access request is denied and makes Track A ineligible.

BR-06 does not claim an adversarial operating-system sandbox. Native code may use ambient OS APIs or deliberate traversal outside the declared interface unless a later platform sandbox or broker prevents it. The BR-06 claim is exact: observer data and paths are not granted through BONSAI-controlled handles, args, environment, working directory, or protocol, and violations detected at those seams fail closed.

The committed matrix is `fixtures/agent-isolation/v1/expected-outcomes.json`.
