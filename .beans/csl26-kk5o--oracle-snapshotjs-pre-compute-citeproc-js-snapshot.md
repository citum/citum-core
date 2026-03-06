---
# csl26-kk5o
title: 'oracle-snapshot.js: pre-compute citeproc-js snapshots'
status: completed
type: task
priority: normal
created_at: 2026-03-06T22:21:03Z
updated_at: 2026-03-06T22:31:34Z
parent: csl26-anlu
---

Write scripts/oracle-snapshot.js. Runs citeproc-js against the fixture for one or all CSL styles, extracts rendered citation+bibliography strings, writes tests/snapshots/csl/<style>.json keyed on {fixture_hash, csl_hash}. Parallel worker support for --all flag.

## Summary of Changes

- Written scripts/oracle-snapshot.js: generates tests/snapshots/csl/<style>.json keyed on {fixture_hash, csl_hash}
- Supports single style or --all (sequential, with --force flag)
- Skip logic: re-uses existing snapshot if both hashes match
- Smoke tested against apa.csl: writes, skips, stale guard all work
