# ADR 0007: Reference science and attribution

- Status: accepted
- Date: 2026-07-18
- Owner: BONSAI maintainers
- Source: approved PSPR v0.1

## D-17 — BONSAI Reference Discovery Cycle v1

Decision: BRDC-1 uses public tabular/linear ingredients: reward-respecting feature-attainment subproblems, options, learned option models, planning, and explicit backward utility credit. It is STOMP/OaK-style and is not an Oak Lab implementation.

Rejected alternatives: a neural mandate is unnecessary for instrument validation; representing unpublished mechanisms as implemented was rejected.

Consequences: names, reports, and documentation retain the attribution boundary; BRDC-1 may fail capability criteria without invalidating the instrument.

## D-18 — Scenario ownership and adapters

Decision: initial coverage uses BONSAI-owned deterministic diagnostic worlds plus procedurally enlarged big-world variants for all ten charter families. The adapter supports external Gymnasium-compatible environments without making Gymnasium a core dependency.

Rejected alternatives: external-only worlds reduce causal control; custom-only interfaces reduce reuse.

Consequences: diagnostic truth and generation provenance are versioned; optional integrations cannot be required for canonical local runs.

## D-19 — Consequential planning backup

Decision: a planning backup is consequential when an identical-state and identical-random-draw counterfactual that omits it changes a later policy distribution beyond declared epsilon or changes an action within the attribution horizon. Approximate influence is labeled and calibrated against exact tabular counterfactuals.

Rejected alternatives: counting any value change would include work that never affects behavior.

Consequences: exact counterfactual fixtures anchor the definition; approximate methods cannot silently inherit exact status.

## D-20 — Utility evidence hierarchy

Decision: exact leave-one-artifact-out or omit-one-event counterfactuals lead in diagnostic worlds; matched ablations support scenario claims; calibrated consumer-credit/influence estimates scale online curation. Proxy utility alone cannot establish C3.

Rejected alternatives: exact counterfactuals everywhere are infeasible; proxy-only credit is scientifically weak.

Consequences: every utility result declares evidence level; claim adjudication requires controlled downstream effect at the prescribed level.

## Supersession

Only a dated, user-approved PSPR addendum and replacement ADR may change reference-agent attribution, scenario obligations, consequential-backup semantics, or utility eligibility.
