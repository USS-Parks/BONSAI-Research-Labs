# BONSAI M0 governed-foundation checkpoint

Status: executing BG-10; implementation checkpoint commit pending  
Authorized scope: BG-01 through BG-10 only  
Source-of-truth basis: approved charter and PSPR v0.1 plus the approved 2026-07-18 public-repository addendum  
Next implementation prompt after M0: BC-01, not authorized by `run M0 STS`

## Repository and publication identity

- Authoritative repository: `https://github.com/USS-Parks/BONSAI-Research-Labs`
- Authorized publication branch: `main`
- Isolated STS branch: `codex/m0-governed-foundation`
- Initial charter revision: `7d0ab846e46a9f38c3bd017da4837bf254b76bdc`
- BG-09 workflow checkpoint: `59e474a8a3eeddbc071b02c0152d8d7925b9af27`
- BG-09 live workflow: `https://github.com/USS-Parks/BONSAI-Research-Labs/actions/runs/29669146969`, all four jobs successful
- Public scope: approved charter history and gated M0 source/documents only; no later release/evidence/capability publication authority

## M0 prompt reconciliation

| Prompt | Outcome | Focused identity/evidence |
|---|---|---|
| BG-01 | independent repository identity passed | `210024f` |
| BG-02 | source-of-truth governance passed | `7193f22` |
| BG-03 | D-01–D-21 ADR coverage passed | `51b9563` |
| BG-04 | dual license/publication policy passed | `482f4d7` |
| BG-05 | locked Rust/Python scaffold passed | `444eb6b` |
| BG-06 | append-only verification machinery passed | `369bad3` |
| BG-07 | risk/blocker/parked ledgers passed | `98ed62c` |
| BG-08 | terminology/identifier/unit registry passed | `85e408d` |
| BG-09 | four-job hosted CI gate passed | `59e474a` plus run 29669146969; closeout `d277415` |
| BG-10 | source-of-truth audit | implementation checkpoint pending |

## Settled decisions and records

- ADR index maps D-01 through D-21 exactly once with no unresolved material decision.
- The 2026-07-18 addendum supersedes only the repository URL and private-visibility part of D-09 for the authorized public scope.
- Rust 1.96.0 and Python 3.12–3.14 surfaces are locked; universal gates are defined and green locally and in hosted CI.
- DEVLOG and verification log contain the executed history, exact gates, source identities, and limitations.
- R-02 and R-03 are resolved. R-13 remains a future physical-host blocker; R-16 remains active for publication. Other initial risks remain active unless explicitly recorded otherwise.
- P-01 through P-09 remain parked. No parked scope was revived.
- C0–C5 and instrument completion remain `not-run`; M0 makes no runtime, measurement, governance-enforcement, physical-host, energy, scientific, or evaluated-agent capability claim.

## Platform evidence

Hosted CI passed on Windows x86_64, Linux x86_64, macOS arm64, and macOS x86_64 at the BG-09 workflow checkpoint. Each artifact records `hosted-ci` and `github-hosted-ephemeral-vm`, with physical acceptance, energy claims, and long-duration claims set false.

Physical Windows/macOS/Linux measurement, enforcement, energy, conformance, acceptance, and long-duration rows remain `not-run` and belong to later milestones.

## M0 completion gate

BG-10 closes only when:

1. `scripts/check_m0.py` and every component governance checker pass.
2. Cargo format, strict Clippy, Rust tests, Ruff, strict Pyright, and Pytest pass.
3. Gitleaks finds no secret in all Git history intended for publication.
4. The BG-10 implementation commit passes the checker from a clean working tree.
5. The closeout commit is pushed without force to the authorized `main` and equals the remote SHA.
6. The final `main` GitHub Actions matrix passes.
7. The DEVLOG, verification log, PSPR statuses, risks, parked scope, and this checkpoint agree.
8. Execution stops before BC-01 because the user authorized M0 only.

Passing M0 establishes a governed foundation. It does not establish a working BONSAI instrument.
