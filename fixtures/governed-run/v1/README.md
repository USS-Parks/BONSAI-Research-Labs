# Governed run verifier fixture

The positive directory preserves a real 20-step WSL2 Linux run produced by the BX-06 runner. Its operator stdout is retained separately in operator-receipt.json and pins the receipt. The source snapshot was dirty; this is diagnostic evidence, with no physical-host or publication eligibility claim.

The fresh Windows verifier reconstructs 380 events, 20 actions/rewards, 9 completed episodes, reward sum 9, and 80 work items. The track declaration retains runtime_facts_complete=false; the verifier derives its own facts. Portable Rust tests mutate copies and rebuild checksums to exercise semantic rejection. The original bytes remain unchanged.
