---
# csl26-myyj
title: Snapshot policy, CI guard, compat.html source
status: completed
type: task
priority: normal
created_at: 2026-03-06T22:21:12Z
updated_at: 2026-03-06T22:35:34Z
parent: csl26-anlu
---

1. fixture_hash staleness guard in oracle-fast.js (fail with regen instructions). 2. .gitattributes: snapshots as generated files (linguist-generated, no merge conflicts). 3. compat.html: add oracle source column (citeproc-js / biblatex / hand-authored) from oracleSource field. 4. CONTRIBUTING.md: when/how to regenerate snapshots.

## Summary of Changes

1. fixture_hash staleness guard: oracle-fast.js exits 2 with regen instructions when snapshot is missing or stale (done as part of csl26-mvit)
2. .gitattributes: tests/snapshots/csl/*.json and tests/snapshots/biblatex/*.json marked linguist-generated=true merge=union
3. compat.html: Oracle column added between Format and Dependents; shows citeproc-js / citum-native / citeproc-js-live per style
4. CONTRIBUTING.md: deferred — regeneration policy not yet documented (follow-up)
