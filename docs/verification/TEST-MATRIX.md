# BONSAI verification test matrix

Status: M0 baseline  
Owner: verification lead  
Rule: hosted CI proves portable semantics and catches regressions; it does not prove physical resource enforcement, energy fidelity, thermal stability, or long-duration acceptance

## Evidence classes

- `hosted-ci` — an ephemeral GitHub-hosted virtual machine. The evidence records the runner label, reported OS/architecture, workflow/run identity, and source revision. `physical_acceptance=false` always.
- `local` — an interactive or automated local run whose physical/virtual status is explicit or `unknown`.
- `physical-host` — a named evidence class available only when a later prompt records sanitized host attestation, capabilities, collector/calibration state, and physical test procedure.
- `simulated` — a test double or fixture. It never closes a live OS, energy, process-control, or physical-host gate.

## M0 hosted baseline

| OS family | Runner label | Architecture | Evidence class | Checks | Physical acceptance | Energy/long-duration claim | Status |
|---|---|---|---|---|---|---|---|
| Windows | `windows-2025` | x86_64 | hosted-ci | Rust/Python universal gates plus governance | no | no | workflow-defined; live run required |
| macOS | `macos-15` | arm64 | hosted-ci | Rust/Python universal gates plus governance | no | no | workflow-defined; live run required |
| macOS | `macos-15-intel` | x86_64 | hosted-ci | Rust/Python universal gates plus governance | no | no | workflow-defined; live run required |
| Linux | `ubuntu-24.04` | x86_64 | hosted-ci | Rust/Python universal gates plus governance | no | no | workflow-defined; live run required |

The workflow pins action commits, uv 0.11.29, Python 3.12, and Rust 1.96.0. Every successful matrix job uploads a JSON classification artifact that explicitly says `hosted-ci`, `github-hosted-ephemeral-vm`, `physical_acceptance=false`, `energy_claim=false`, and `long_duration_claim=false`.

## Later physical-host obligations

| Evidence | Windows 11 x86_64/MSVC | Apple-silicon macOS | x86_64 Linux/cgroup v2 | Earliest closing prompt | M0 status |
|---|---|---|---|---|---|
| Adapter/process physical spot check | required | required | required | M1/M3 integration prompts | not-run |
| Live resource calibration | required | required | required | BM-05–BM-14 | not-run |
| Hard/monitor-terminate budget violation | required | required | required | BQ-07 / M3 | not-run |
| Energy E0–E3 | capability and tier explicit | capability and tier explicit | capability and tier explicit | BM-12, BM-13, BV-06 | not-run |
| Acceptance A profile | required for C4/C5 candidacy | required | required | BE-16 / BV-06 | not-run |
| Long L profile | 72 h / 10 M steps / 3 paired seeds | same | same | BV-14 | not-run |
| Offline restore | clean physical host or isolated VM | same | same | BV-13 | not-run |

No hosted result may update these `not-run` rows to pass.

## BG-09 gate

BG-09 closes only after a no-op-equivalent source push causes all four hosted jobs to complete successfully and their generated evidence artifacts identify runner virtualization and evidence class. The DEVLOG records the workflow run URL and job conclusions. A workflow file that merely parses locally is insufficient.
