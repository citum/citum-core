---
# csl26-ft25
title: 'Fix benchmark pass semantics: add min_pass_rate threshold'
status: completed
type: bug
priority: high
created_at: 2026-04-11T11:04:13Z
updated_at: 2026-04-11T11:20:19Z
---

The citeproc-oracle benchmark run sets status='pass' whenever the oracle runs without crashing, regardless of match rate. A 73% match rate (292/400) gets a green 'pass' badge. Fix: add min_pass_rate field to benchmark run config. When set, status becomes 'pass'/'fail' based on match rate vs threshold; when absent, status becomes 'ok' (neutral) to distinguish 'ran OK' from 'actually passed a quality bar'. Update HTML renderer accordingly.

## Summary of Changes

- Added `min_pass_rate` field to benchmark run config in `scripts/report-data/verification-policy.yaml` and validation in `scripts/lib/verification-policy.js`
- Changed `runBenchmarkRun` status logic from binary pass/error to tri-state: `pass` (rate >= threshold), `fail` (rate < threshold), `ok` (no threshold set, ran without error)
- Updated HTML renderer status badge: emerald for pass, slate for ok, red for fail/error
- Set `min_pass_rate: 0.73` on `chicago-zotero-bibliography` benchmark run
