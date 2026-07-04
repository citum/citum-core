---
# csl26-ol1j
title: Silent-fallback warning batch (part 2)
status: todo
type: task
tags:
    - warnings
    - rendering
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

Render-time silent fallbacks that merit structured warnings: missing locale term renders empty (values/term.rs unwrap_or_default); SelectorEvaluator::matches_field returns false for any field except language/note; numeric-collapse hardcodes en-dash/comma with no locale hook; format_role_term honors only capitalize-first and drops other declared text-case transforms. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 22 (remainder).
