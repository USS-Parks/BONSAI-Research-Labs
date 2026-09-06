# PSPR v0.5 Project Handoff

Date: 2026-09-05 (America/Los_Angeles)
Status: **APPROVED — FULL BX-01–BX-40 STS AUTHORIZED; BX-01–BX-02 VERIFIED; BX-03 EXECUTING**

## Current execution authority — 2026-09-05

> Approved for STS now. Authorization to run this until verified complete, committing and merging with main after each prompt is finished fully, and all 40 prompts are successfully executed.

The approved project plan section 14 records this authority. Begin/resume BX-01 and continue sequentially after each prescribed gate and commit/main publication. The sections below preserve the earlier import snapshot; their pending-approval statements are historical.

## Review document and authority

[PSPR v0.5-draft.1](./BONSAI-PSPR-v0.5-EXECUTABLE-RESEARCH-AND-GOVERNED-EVOLUTION.md) is the single project-local review copy. It was imported byte-for-byte from the user-supplied attachment; SHA-256: `af7d66762d3d77569a8f47e774ff9924cd31a474c366dc45b02878170fe19f7e`.

Importing the draft into its proposed repository location is document integration only. It does not adopt the plan as approved, supersede the approved v0.1–v0.4 history, start BX-01, or authorize source publication. All BX-01–BX-40 remain NOT STARTED. No approval or STS wording accompanied the attachment.

The draft's references to its original outputs folder describe its source-session history. Use the linked project copy for review here. The companion architecture review and original transfer note named in section 13 were not included in the supplied attachment; this handoff does not recreate or claim to verify them. Their supporting review evidence must be supplied or independently re-established during authorized BX-01.

## Reconciliation at import

- Checked local main: `2f35355c6d12d019eb8625cb3bd38728d90ee029`, matching the cached origin/main. The working tree was clean before document integration.
- The earlier user-authorized commit/merge task fast-forwarded this checkout to that SHA. The draft's older local HEAD and behind counts in sections 0.3 and 8 are preserved historical observations, not current checkout status. No checkout update was performed for this import.
- The [canonical PSPR](./BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md) remains the parent governance record. [M4](./BONSAI-PSPR-v0.4-M4-ACCEPTANCE-AND-RELEASE.md), the [DEVLOG](../docs/sessions/BONSAI-DEVLOG.md), and the [verification log](../docs/verification/BONSAI-VERIFICATION-LOG.md) retain their recorded implementation history and physical-host exceptions. This import does not revalidate their scientific or live-system claims.
- The canonical checkout remains the active lane. The historical `C:/tmp/bonsai-m0-sts` worktree remains at `eaa0e52ec5a6dc78ab1a360f2a11c2201c7a5e9d`; the preceding merge task verified it clean with no unpublished commits relative to main and approximately 1.36 GiB of target/virtual-environment data. It is retained pending explicit removal authorization. Canonical generated-data size remains partially unreadable; the draft's 8.15 GiB lower bound is historical, not a fresh measurement.
- The inherited AGENTS.md references RTK.md, but `C:/Users/17076/RTK.md` was absent at import. No RTK-specific instructions were inferred.

## Next step

Review v0.5 and record any requested changes. The draft recommends M5a (BX-01–BX-06) as the first usable cut. Plan approval alone does not start execution. After explicit approval and scoped STS authorization, BX-01 is first and must revalidate the baseline, review findings, full applicable gates, hosts, and resource availability.

Use the existing DEVLOG and verification log. Preserve one prompt per focused commit, the M5a–M5f dependency order, all gates, and the publication/host boundaries in the draft. No new worktree, experiment, implementation commit, or push is part of this import.

## Current continuation — BX-02

BX-01 closed at `51637ee2dfb3b21673bfeddecb8533a8dd6e29d5`, hosted run 33998986034 fully green. BX-02 is executing. User-local WSL Rust 1.96.0 is installed without sudo/profile changes; the three existing process_transport tests pass on x86_64-unknown-linux-gnu. Reuse the canonical target root with the Linux target triple and limit build jobs to four. Windows .venv remains intact; WSL subprocess fixtures use system Python.

## Current continuation — BX-03

BX-02 closed at `9a2457afd873bac167ffd61602b64f624f7ea089`, hosted run 33999964570 fully green. BX-03 is executing against the existing unprivileged WSL2 systemd delegation. The supervisor subgroup prerequisite was verified; no global controller changes or physical-host claim.
