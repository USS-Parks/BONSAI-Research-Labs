# Platforms

Windows, macOS, and Linux are first-class CI targets. Hosted runners are ephemeral VMs.

| Evidence | What it is | What it is not |
|---|---|---|
| Hosted CI baseline | Portable semantics and regressions | Physical Job Object, cgroup hard limit, Apple energy, or 72 h L |
| Container cgroup read | Measurement when `/sys/fs/cgroup` is readable | Delegated-cgroup enforcement |
| L harness + not-run attestations | D-16 minima are declared | A 72-hour physical pass |

See [TEST-MATRIX](../verification/TEST-MATRIX.md), [Windows Job Object](../architecture/WINDOWS-JOB-OBJECT.md), [macOS backend](../architecture/MACOS-RESOURCE-BACKEND.md), [Linux cgroup](../architecture/LINUX-CGROUP-BACKEND.md), and [energy tiers](../metrics/ENERGY-TIERS.md).

Do not overlay incomparable platforms or treat one short CI job as L acceptance (OD-01).
