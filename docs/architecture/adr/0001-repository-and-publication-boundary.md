# ADR 0001: Repository and publication boundary

- Status: accepted
- Date: 2026-07-18
- Owner: BONSAI maintainers
- Source: approved PSPR v0.1

> Amendment: the approved [2026-07-18 public-repository addendum](../../governance/addenda/2026-07-18-public-repository-target.md) supersedes the repository URL and private-visibility portion of D-09 for gated charter and M0 source publication. The decision text below is retained as PSPR v0.1 history.

## D-01 — Independent repository

Decision: BONSAI uses an independent repository rooted at `C:\Users\17076\Documents\Reinforcement Learning Project`. It does not attach to, import, or share history with any parent repository.

Rejected alternatives: a different root and a monorepo were rejected because they change provenance, worktree isolation, release identity, and evidence paths.

Consequences: Git top-level identity is a gate; concurrent STS sessions use isolated worktrees; evidence records the exact source revision and dirty state.

## D-09 — License, visibility, and publication

Decision: source is dual-licensed Apache-2.0 OR MIT. The repository remains private through final evidence audit. Public visibility and external release are separate, explicit publication decisions.

Rejected alternatives: a single license was not selected; public-by-default development and premature artifact publication were rejected because they increase legal, privacy, and claim risk.

Consequences: both license texts and SPDX expression are required; contribution and publication policy must preserve the private boundary; external contributions or uploads require review and authority.

## Supersession

Only a dated, user-approved PSPR addendum and replacement ADR may supersede these decisions. The replacement must identify repository/provenance migration and publication consequences.
