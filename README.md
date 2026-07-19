# BONSAI

**Benchmark for Online, Nonstationary, Single-pass Agent Intelligence**

BONSAI is an independent, algorithm-neutral measurement and external resource-governance instrument for online, nonstationary, single-pass agents. It observes OaK-style discovery cycles; it is not an implementation of unpublished Oak Lab algorithms.

## Governance status

- Research charter: approved v0.1 on 2026-07-18.
- PSPR: approved v0.1 on 2026-07-18.
- Current execution authorization: M0 (BG-01 through BG-10) only.
- Implementation claims: none. M0 creates the governed foundation, not a functioning instrument.
- Repository visibility: private unless a separately approved publication action changes it.

The approved PSPR is the execution roster, but approving or editing it is not authorization to execute work. Implementation may begin only after the user says `run it STS`, `run M0 STS`, or explicitly authorizes named prompts. External publication, visibility changes, privileged collectors, credentials, and destructive actions require their own authority.

## Repository identity

- Authoritative repository: `https://github.com/USS-Parks/BONSAI`
- Authoritative local root: `C:\Users\17076\Documents\Reinforcement Learning Project`
- Default branch: `main`
- M0 STS branch: `codex/m0-governed-foundation`
- Parent repository history: excluded

The M0 session runs in an isolated Git worktree. The authoritative checkout and the isolated worktree share this repository's object database but never share a Git index.

## Sources of truth

Start with the [BONSAI Research Charter package](./BONSAI%20Research%20Charter/README.md), including the [research charter](./BONSAI%20Research%20Charter/BONSAI-RESEARCH-CHARTER.md), [OaK evidence register](./BONSAI%20Research%20Charter/OAK-EVIDENCE-AND-TRACEABILITY.md), and [approved PSPR](./BONSAI%20Research%20Charter/BONSAI-CANONICAL-PLAN-SEQUENTIAL-PROMPT-ROSTER.md).

## Baseline identity record

The pre-STS reconciliation on 2026-07-18 established:

- authoritative checkout top level: `C:/Users/17076/Documents/Reinforcement Learning Project`;
- baseline revision: `7d0ab846e46a9f38c3bd017da4837bf254b76bdc`;
- `main` and `origin/main` both resolved to that revision;
- the working tree was clean;
- the index contained only the six approved charter-package documents;
- no indexed path referred to `C:\Users\17076` or any parent repository;
- the isolated M0 worktree was created from that exact revision.

Prompt-level commands and results are retained in the verification log once BG-06 establishes it.
