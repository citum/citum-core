---
# csl26-6ycw
title: Support multiple --abbreviation-map flags on CLI
status: todo
type: feature
priority: low
created_at: 2026-05-13T11:47:44Z
updated_at: 2026-05-13T11:47:47Z
blocked_by:
    - csl26-zv1y
---

Allow citum render to accept --abbreviation-map more than once. Maps are merged in order of appearance; later entries win on key collision. Enables scholars to compose domain-specific abbreviation files (e.g. a journals file plus a legal-bodies file) without pre-merging. Engine DocumentOptions.abbreviation_map stays as a single AbbreviationMap; merging is a CLI concern only.
