# BONSAI risk and blocker register

Owner for the register: BONSAI maintainers  
Review cadence: every milestone checkpoint, before each affected prompt, and immediately after a trigger or scope change  
Status vocabulary: `active`, `future-blocker`, `blocked`, `mitigated`, `resolved`, `accepted`

| ID | Type | Risk or blocker | Owner | Trigger | Impact | Mitigation | Status | Affected prompts | Review cadence |
|---|---|---|---|---|---|---|---|---|---|
| R-01 | risk | OaK mechanisms remain publicly underspecified | Scientific lead | Reference work could be described as reproduction | Misattribution and invalid scientific claim | Preserve evidence labels and BRDC-1 boundary | active | BG-02, BE-04, BV-15 | Before reference-agent or publication work |
| R-02 | blocker | Git could resolve to an unrelated parent repository | Repository maintainer | Top level or common Git directory differs from BONSAI identity | Wrong history, commits, or remote | Independent repository gate and isolated worktrees | resolved | BG-01 | Every session start |
| R-03 | blocker | D-01 through D-21 might be unresolved | Architecture owner | ADR coverage has an unresolved material decision | Dependent prompts lack authority | Approved v0.1 decisions and exact ADR coverage check | resolved | BG-03 and all dependent prompts | On every addendum |
| R-04 | risk | OS resource primitives are not semantically uniform | Platform lead | Capability or calibration differs by host | False matched-budget or equivalence claim | Capability model, spikes, fail-closed preflight | active | BM-05–BM-14, BQ-07 | Before platform gates |
| R-05 | risk | Energy is unavailable, shared, low-resolution, or privileged | Measurement lead | Collector cannot establish the required tier | False zero/precision or C5 overclaim | E0–E3 availability and calibration rules | active | BM-12, BM-13, BV-06 | Before any energy claim |
| R-06 | risk | Instrumentation changes agent timing or behavior | Measurement lead | Paired overhead crosses D-11 or changes outcomes | Invalid comparisons | Paired overhead measurement and acceptance ceiling | active | BM-04, BK-03, BV-10 | Every instrumentation epoch |
| R-07 | risk | Observer telemetry becomes hidden replay | Runtime lead | Agent can access retained telemetry or derived history | Track A invalidation | Process/data isolation and derived classification | active | BR-06, BR-09, BQ-06 | Every adapter and track change |
| R-08 | risk | Faulty adapters flood events/artifacts or fork descendants | Security lead | Rate, size, process, or descendant bounds exceeded | Denial of service or escaped budget | Bounded transport, ingest, and process controls | active | BR-02, BR-03, BQ-07, BV-12 | Before untrusted adapter runs |
| R-09 | risk | Telemetry exceeds observer budget | Ingestion lead | Required event/output volume crosses policy | Evidence loss or biased sampling | Segmentation, backpressure, reserve, bounded schemas | active | BC-09, BQ-05, BV-12 | Every event/schema epoch |
| R-10 | risk | Cross-platform nondeterminism or floating-point drift | Reproducibility lead | Semantic fixture exceeds declared tolerance | Reproducibility failure | Semantic tolerances, declared nondeterminism, paired seeds | active | BK-14, BV-09 | Every supported platform change |
| R-11 | risk | Ablation and seed matrix becomes infeasible | Experiment lead | Planned matrix exceeds declared profiles/resources | Incomplete claims or hidden pruning | Profiles, pilots, preregistration, power review | active | D-15, D-16, BE-16 | Before C, A, or L runs |
| R-12 | risk | Utility proxy rewards activity instead of downstream effect | Metrics lead | Proxy is used without exact/matched calibration | Spurious C3 or C4 | Exact diagnostics and proxy ineligibility alone | active | BK-10, BV-05 | Every utility metric version |
| R-13 | blocker | Required long-duration physical hosts may be unavailable | Acceptance owner | A required Windows, macOS, or Linux host is not committed | M4 and release completion blocked | Secure hosts before BV-14; never substitute CI | future-blocker | BV-14 | At M3 exit and before BV-14 |
| R-14 | risk | Privileged collectors broaden attack surface | Security and measurement leads | A collector requests elevated access | Integrity or host-security exposure | Opt-in, least privilege, threat model, explicit authority | active | BM-13, BV-11, BV-12 | Before privileged collector use |
| R-15 | risk | Dependency or format churn breaks old bundles | Schema lead | Locked dependency or schema epoch changes | Long-term audit failure | Migrations, compatibility fixtures, offline archives | active | BC-01, BC-12, BV-13 | Every schema/dependency change |
| R-16 | risk | Public release exposes secrets, private material, or unsupported claims | Publication owner | Any external push, upload, or announcement | Legal, privacy, security, or reputational harm | Target-specific authority, redaction/secret scan, claim audit | active | BG-04, BG-10, BV-16 | Immediately before every publication |

Resolved risks remain in the register as history. Status changes require a dated note in the DEVLOG with evidence and affected prompt IDs.
