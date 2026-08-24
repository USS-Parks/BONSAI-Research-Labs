# Claims

Machine rules emit C0–C5. Operators do not paint cells green.

- [C0/C1](../claims/C0-C1-ADJUDICATION.md)
- [C2/C3](../claims/C2-C3-ADJUDICATION.md)
- [C4/C5](../claims/C4-C5-ADJUDICATION.md)
- [Claim-to-evidence matrix](../governance/CLAIM-TO-EVIDENCE-MATRIX.md)

Rules:

1. A missing prerequisite is `indeterminate` or `not-run`, never pass.
2. Tamper fails C0. An unavailable declared hard counter cannot pass C1.
3. Proxy-only utility cannot pass C3. C4/C5 shortcuts cannot pass.
4. Reproducing M1 is not an evaluated-cycle pass.
5. BRDC-1 is not an unpublished Oak Lab / OaK reproduction.

Publication of any claim still needs a separate authorization phrase.
