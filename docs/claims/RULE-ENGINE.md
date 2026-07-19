# Claim-ladder rule engine

BV-01 encodes one versioned, agent-neutral C0–C5 rule graph. Each level names prerequisite levels, criteria, evidence tiers, and exact fact IDs. Evaluation returns pass, fail, or indeterminate plus a sorted reason graph and stores the rule version in every result.

Explicitly failed facts fail their criterion. Missing or contradictory facts are indeterminate. Higher levels cannot bypass a failed or indeterminate prerequisite. The skeleton does not grant a reference agent a pass; later BV prompts strengthen the criteria with concrete evidence rules.
