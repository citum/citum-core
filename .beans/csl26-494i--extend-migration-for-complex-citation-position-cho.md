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
updated_at: 2026-03-10T18:29:03Z
---

Current implementation maps CSL position branches only when citation layout is a single root <choose> with position-only conditions.\n\nFollow-up:\n- support nested/non-root position chooses without dropping sibling content\n- handle multiple position chooses in one citation layout\n- keep explicit warning behavior for truly unsupported mixed condition trees\n- add migration fixtures covering these shapes.
