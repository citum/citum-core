---
# csl26-ebs3
title: Year-suffix disambiguation for undated works
status: completed
type: task
priority: normal
tags:
    - dates
    - sorting
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-05T23:02:25Z
parent: csl26-8m2p
---

Two undated works by the same author both render (Smith, n.d.) — the no-date path returns the term before the disambiguation suffix is computed and compute_disamb_suffix requires a non-empty year, though grouping already produces the hints. Apply the suffix to the no-date term (APA: n.d.-a) with a locale/style-controllable joiner. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 7.

## Summary of Changes\n\nApplied year-suffix disambiguation to issued no-date fallback terms, defaulting to APA-style hyphen joins such as n.d.-a and supporting a configurable date option for the no-date suffix delimiter. Added citation behavior tests, date option parsing/default tests, and regenerated the style schema.
