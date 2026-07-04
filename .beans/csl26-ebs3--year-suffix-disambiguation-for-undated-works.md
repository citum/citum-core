---
# csl26-ebs3
title: Year-suffix disambiguation for undated works
status: todo
type: task
tags:
    - dates
    - sorting
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

Two undated works by the same author both render (Smith, n.d.) — the no-date path returns the term before the disambiguation suffix is computed and compute_disamb_suffix requires a non-empty year, though grouping already produces the hints. Apply the suffix to the no-date term (APA: n.d.-a) with a locale/style-controllable joiner. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 7.
