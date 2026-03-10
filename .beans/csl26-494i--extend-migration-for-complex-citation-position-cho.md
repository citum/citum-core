---
# csl26-494i
title: Extend migration for complex citation position choose trees
status: todo
type: task
priority: normal
tags:
    - migration
    - citations
created_at: 2026-03-10T18:29:03Z
updated_at: 2026-03-10T19:40:36Z
---

Current implementation maps CSL position branches only when citation layout is a single root <choose> with position-only conditions.

Follow-up:
- support nested/non-root position chooses without dropping sibling content
- handle multiple position chooses in one citation layout
- keep explicit warning behavior for truly unsupported mixed condition trees
- add migration fixtures covering these shapes.
