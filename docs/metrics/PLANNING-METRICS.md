# Planning metrics and consequential backups

BK-09 measures planning updates, states and options considered, search-control operations, value gain per operation, realized agreement, primitive-time depth, latency, backups saved, exploitation failure, and unused plans.

A backup is **consequential** only under D-19: a paired omit-one counterfactual with identical pre-backup state and random draws must change a later policy distribution beyond declared `epsilon` or change an action. A value change is recorded and is not sufficient. Approximate influence estimates are labeled and report absolute error against the exact tabular counterfactual.

The committed fixture at `fixtures/planning-metrics/v1/expected-outcomes.json` identifies value-only, policy-shifting, action-changing, and mis-calibrated approximate backups.
