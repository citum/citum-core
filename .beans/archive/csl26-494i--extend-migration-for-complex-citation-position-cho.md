---
# csl26-494i
title: Extend migration for complex citation position choose trees
status: completed
type: task
priority: normal
tags:
    - migration
    - citations
created_at: 2026-03-10T18:29:03Z
updated_at: 2026-03-10T19:55:18Z
---

Implemented on branch `codex/csl26-494i-complex-position-choose`.

Delivered:
- nested/non-root position chooses now migrate without dropping sibling content
- multiple position chooses in one citation layout are merged into full first/subsequent/ibid variants
- unsupported mixed-condition trees still warn and fall back to the base citation template
- migrate regression coverage now covers nested, multi-choose, and unsupported shapes

Follow-on handoff lives in `csl26-qfa3`.
