# Metric registry

BK-01 makes metric ID, version, formula, unit, window, direction, exact inputs, availability rule, and claim use explicit before computation. Every dependency reference pins both ID and version. The registry rejects missing or mis-versioned inputs, duplicate definitions, malformed formulas, and cycles.

The engine evaluates a deterministic dependency order using checked normalized rational arithmetic. Missing source evidence propagates `unavailable` rather than numeric zero. Output rows are key-sorted and serialize identically across operating systems. Reports may display only registered table values; they may not introduce unregistered calculations.
