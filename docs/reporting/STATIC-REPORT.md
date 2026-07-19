# Static report contract

BV-02 produces `bonsai.static-report/v1` machine JSON and a self-contained HTML view from one typed payload. The HTML is rendered only after the machine JSON has been serialized and parsed back, so it has no separate calculation path. Manifest, platform, track, resource, overhead, behavior, failure, claim, limitation, and hash values remain visible and attributable to the machine record.

The report contains inline presentation rules but no scripts, links, external assets, or network requests. It uses language, title, main, heading, caption, column-header, and row-header semantics for offline accessibility. Missing required sections and malformed content hashes fail closed. A generated report is a view of supplied metrics and verdicts; it does not compute metrics or create a C0–C5 claim.
