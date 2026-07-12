---
# csl26-i41r
title: Bibliography entry-separator option
status: todo
type: task
priority: low
tags:
    - schema
    - rendering
created_at: 2026-07-12T18:15:46Z
updated_at: 2026-07-12T18:15:46Z
parent: csl26-kcda
---

CSL schema#386: no bibliography entry-separator/newline-suppression option
found in the style.json schema surface. Lower confidence than sibling
beans from this triage — needs confirming against citum-engine's
per-output-format bibliography renderers directly (not just the schema),
since this may be a render-layer concern the schema wouldn't necessarily
expose either way.

- [ ] Engine-side check: how do per-output-format bibliography renderers
      currently join entries, and is a separator override even missing
- [ ] If confirmed missing, design a bibliography entry-separator option
