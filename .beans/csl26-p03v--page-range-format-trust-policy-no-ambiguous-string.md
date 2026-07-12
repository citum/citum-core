---
# csl26-p03v
title: Page-range-format trust policy (no ambiguous-string expansion)
status: todo
type: task
priority: low
tags:
    - schema
    - rendering
created_at: 2026-07-12T18:15:22Z
updated_at: 2026-07-12T18:15:22Z
parent: csl26-kcda
---

CSL schema#81: no documented policy on trusting vs "expanding" ambiguous
page-range input strings (e.g. "110-5" could mean "110-115" or could be a
leaf-number pinpoint that should not be touched). Needs a design decision
(trust user input verbatim vs current expansion heuristic), not just an
implementation.

- [ ] Design: trust-user-input vs expand-heuristic policy for page ranges
