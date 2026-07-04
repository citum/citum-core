---
# csl26-mc0c
title: Contributor spec divergences (oracle-first)
status: todo
type: task
tags:
    - contributors
    - types
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

Batch of silent spec divergences in values/contributor that override declared options; each needs oracle fixtures BEFORE changing behavior: (a) delimiter-precedes-last hardcoded branches (two-name citation 'never', GivenFirst bibliography 'never', Contextual true for two names); (b) substitute-title always quoted in citation context, bypassing per-type title formatting; (c) long-form roles auto-append (ed.)-style labels for seven hardcoded roles with no declarative off-switch, and resolve_explicit_label silently substitutes the role term for unknown keys; (d) inverted-name suffix joins with space instead of sort-separator (Smith, J. Jr. vs Smith, J., Jr.). docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md findings 9, 16, 19.
