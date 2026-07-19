# Resource sample interface

BM-01 defines the platform-neutral boundary between collectors and the rest of BONSAI. It does not claim that any counter is available on every operating system.

Each sample identifies a counter, resource kind, scope, monotonic sampling time, status, provenance, resolution, uncertainty, and unit-bearing value. Collector outputs have exactly four states:

- `measured`: a backend read a value directly;
- `estimated`: a named estimator produced a value and uncertainty;
- `unavailable`: the backend cannot supply the counter under the current platform or privilege state;
- `error`: a supported read was attempted and failed.

`value: 0` is a real value. Missing or failed measurements carry no value and require a stable detail code. This prevents unavailable CPU, memory, storage, I/O, accelerator, operation, or energy evidence from being interpreted as zero consumption.

Scopes distinguish the agent process, agent process tree, agent-owned storage, observer, accelerator device, and system. A backend must return a validated outcome for each counter it advertises; support is never inferred from an empty result.

This contract is evidence plumbing only. Platform backends, calibration, enforcement, energy tiers, and cross-platform comparability remain later prompts.
