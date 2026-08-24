# Security policy

BONSAI v1 is designed to tolerate faulty adapters through bounded messages, process separation, least privilege, OS resource controls, and fail-closed termination. It is not a hostile-native-code sandbox.

The BV-11 [threat model](docs/security/THREAT-MODEL.md) maps every trust boundary to controls, tests, residual risk, and prompt IDs. Read the D-21 disclaimer there before running untrusted native adapters.

## Reporting

The source repository is public. Do not include credentials, live exploit secrets, personal identifiers, machine serials, or unrelated user data in issues or patches. Prefer a private channel to the repository owner when a report would create immediate host or evidence-integrity risk.

Include the affected revision, platform, minimal reproduction, impact, and whether evidence integrity or observer-to-agent isolation is affected. Coordinate any disclosure or publication separately.

## Supported state

Before a release exists, only the current private development revision is eligible for fixes. Security support does not imply a production-safety, adversarial-sandbox, or autonomous-control certification.
