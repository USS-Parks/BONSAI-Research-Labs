# C0 and C1 adjudication

BV-04 adjudicates C0 (valid provenance, event, and resource evidence) and C1 (enforceable budget compliance with availability qualifications). Soft degradation remains a C1 pass. Hard budget violations fail C1. Tampered evidence fails C0. Missing resource evidence or an ambiguous track is indeterminate. A declared hard counter that was unavailable cannot produce a C1 pass.

The committed fixture at `fixtures/c0-c1-adjudication/v1/expected-outcomes.json` freezes those six bundle classes.
