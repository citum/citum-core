---
# csl26-3c90
title: Semantic sort fallback for missing-name items in alphabetical bibliographies
status: completed
type: bug
priority: high
created_at: 2026-03-08T00:33:31Z
updated_at: 2026-03-08T16:15:00Z
---

Recent sorting changes now fall back to title-based behavior for works that have
no author-like contributor in alphabetical bibliographies. That prevents
unnamed works from clustering incorrectly.

This bean is resolved by the engine change in `6442bd2` and the policy
adjudication recorded as `div-004` in
`docs/adjudication/DIVERGENCE_REGISTER.md`, which makes the title fallback an
intentional divergence from legacy CSL/citeproc behavior rather than an open
bug.

Follow-up style verification moved to `csl26-kafu`.

## Summary of Changes

Archived as completed after confirming the underlying behavior was already
implemented in `fix(engine): sort missing-name works by title` and formally
documented in the divergence register as biblatex-aligned behavior. No further
engine change is required under this bean.
