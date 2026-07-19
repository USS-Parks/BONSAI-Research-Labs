# Lineage graph and causal queries

Status: BR-08 analysis contract
Source: one validated BR-07 registry snapshot

## Boundary

The `LineageGraph` is a read-only projection over recorded provenance edges. It
answers ancestry, descendants, active consumers, exact revision ownership and
history, utility-entry sources, and cost rollups. A returned relationship means
only that an accepted lifecycle event declared it. The query layer does not infer
causality from correlation, timing, shared consumers, utility magnitude, or
representation similarity.

## Validation

Graph construction independently checks the public snapshot before serving a
query:

- each artifact map key agrees with the record's stable identity;
- revision identifiers are globally unique and the owner index is exact;
- every revision names the immediately previous immutable revision;
- changing a representation under an existing revision ID is a distinct,
  fail-closed error;
- every parent names an existing artifact and exact revision;
- the provenance graph is acyclic.

The stable failure codes distinguish duplicate revisions, silent representation
changes, invalid revision chains/owners, dangling edges, and cycles. Unknown
artifact queries fail explicitly rather than returning a misleading empty set.

## Query semantics

Ancestry and descendant results are transitive and ordered by artifact ID.
Revision history remains in lifecycle order. Active consumers are returned from
the exact BR-07 view. Utility sources expose retained metric identity, estimate,
availability, estimator identity, source-event IDs, and method IDs without
assigning causal weight.

Cost rollups support either the named artifact or its descendant subgraph.
Measured and estimated amounts are accumulated separately with checked integer
arithmetic. Unavailable and excluded entries are counted, not converted to zero.
All queries operate on the graph's private snapshot copy and cannot mutate the
authoritative registry or lifecycle events.
