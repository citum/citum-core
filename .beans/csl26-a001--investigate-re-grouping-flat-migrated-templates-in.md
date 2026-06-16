---
# csl26-a001
title: Investigate re-grouping flat migrated templates into authored groups
status: draft
type: task
priority: normal
tags:
    - migrate
    - authorability
    - template
    - fidelity
created_at: 2026-06-16T12:55:29Z
updated_at: 2026-06-16T13:06:37Z
---

Occurrence-compiler emits a flat union template; component groups (imprint = publisher+place, locator = volume+issue+pages) are mostly gone in migrated output (see ACME). Groups are the primary mechanism for conditional formatting: a group renders its delimiter, affixes, and punctuation only when at least one member produces output. Without groups, components render individually — a delimiter or label before an absent variable becomes a stray artifact. Regrouping flat templates is therefore essential for correct output, not just visual tidiness.

GATE ON FIDELITY: confirm the engine renders identically before/after re-grouping. Flatness may be load-bearing for current pass rates. Draft until a safe grouping heuristic is validated.

Authorability follow-up from ACME review (PR #932). Pre-existing; not a regression.
