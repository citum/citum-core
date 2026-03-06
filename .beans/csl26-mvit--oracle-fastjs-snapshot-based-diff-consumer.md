---
# csl26-mvit
title: 'oracle-fast.js: snapshot-based diff consumer'
status: completed
type: task
priority: normal
created_at: 2026-03-06T22:21:05Z
updated_at: 2026-03-06T22:31:34Z
parent: csl26-anlu
---

Write scripts/oracle-fast.js. Loads pre-computed snapshot, diffs Citum output against it. Fails with actionable message if snapshot missing or stale (fixture_hash mismatch). Drop-in replacement for oracle.js in report-core.js non-migrate path.

## Summary of Changes

- Written scripts/oracle-fast.js: loads snapshot, diffs Citum output against it
- Reuses renderWithCslnProcessor from oracle.js (already exported)
- Same comparison logic as oracle.js (equivalentText, matchBibliographyEntries, strict citation IDs)
- Staleness guard: exits 2 with actionable regen message
- Adds oracleSource: 'citeproc-js' to JSON output
- Smoke tested: 18/18 citations, 32/32 bibliography on apa
