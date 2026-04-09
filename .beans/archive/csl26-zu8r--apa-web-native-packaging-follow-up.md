---
# csl26-zu8r
title: apa web-native packaging follow-up
status: completed
type: task
priority: high
created_at: 2026-04-09T15:40:00Z
updated_at: 2026-04-09T18:45:00Z
---

Continue the APA rich bibliography closure pass by fixing the bounded
web-native packaging cluster in `apa-test-library-diagnostic`.

Current verified state:
- baseline APA gate remains `40 / 40`
- supplemental APA diagnostic benchmark improved from `41 / 74` to `44 / 74`
- this bean owns rows `42`, `43`, and `45`
- target references: blog post, forum post, and webpage with part-title /
  editor / translator packaging

Expected owning subsystem:
- primary: `citum_engine`
- secondary: style YAML if the data is already present and APA template assembly
  is still wrong
- migration only if webpage part-title or role metadata is dropped before
  rendering

Current mismatch shape:
- retrieval-date fallback is being injected where APA expects direct URL output
- website title casing / container packaging differs from oracle output
- webpage part-title and inline editor / translator packaging do not match APA

Completed work:
- reduced-cluster oracle moved from `0 / 3` to `3 / 3`
- preserved webpage `part-title` intake from note-field hacks
- synthesized webpage title packaging from base title + part number +
  part title
- added APA-specific `post` and `webpage` bibliography variants to stop
  retrieval-date fallback on ordinary web-native rows
- added a focused regression in `crates/citum-engine/tests/bibliography.rs`

## Acceptance
- rows `42`, `43`, and `45` match the oracle exactly
- baseline APA remains `40 / 40`
- no new regressions appear outside this cluster

## Stop-Loss Rule
- Stop after 2 distinct implementation attempts with no net gain.
- Reclassify immediately as `style-defect`, `processor-defect`, or
  `migration-artifact` and hand off the unresolved rows explicitly.
