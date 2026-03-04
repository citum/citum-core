---
# csl26-o18j
title: Fix missing components in bibliography
status: completed
type: bug
priority: high
created_at: 2026-02-07T18:20:10Z
updated_at: 2026-02-27T01:09:27Z
parent: csl26-ifiw
---

Templates missing critical components: containerTitle (6 occurrences), doi (5 occurrences). These components exist in CSL 1.0 output but are not being included in Citum templates during migration. Investigate template_compiler component selection logic.

## Completion Notes (2026-02-27)

Closed as no-longer-reproducible after current oracle verification:

- `node scripts/oracle-batch-aggregate.js styles-legacy/ --top 10`
  - No `containerTitle:missing`
  - No `doi:missing`
- `node scripts/oracle-batch-aggregate.js styles-legacy/ --top 50`
  - No `containerTitle:missing`
  - No `doi:missing`

Current top bibliography issue is `publisher:extra`, not missing `containerTitle`/`doi`.
Remaining bibliography deltas continue under parent epic `csl26-ifiw`.
