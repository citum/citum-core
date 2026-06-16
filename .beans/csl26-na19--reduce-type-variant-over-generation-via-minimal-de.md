---
# csl26-na19
title: Reduce type-variant over-generation via minimal default template
status: draft
type: task
priority: normal
tags:
    - migrate
    - authorability
    - template
created_at: 2026-06-16T12:55:29Z
updated_at: 2026-06-16T12:55:29Z
---

Migration emits many type-variants that only 'remove' components (e.g. speech extends article-newspaper removing edition/section). Driven by maximal-default + diff-everything model.

Note: term: literals (edition/section labels) render regardless of data presence, so THOSE removes are semantically real. The deeper issue is over-generation: derive a minimal type-agnostic default and have types add what they need, rather than subtract from a maximal union.

Large; converter-dominated tail (see crates/citum-migrate/CLAUDE.md). Draft.

Authorability follow-up from ACME review (PR #932). Pre-existing; not a regression.
