---
# csl26-7auw
title: Improve diff-encoding readability for extends chains
status: todo
type: task
priority: low
tags:
    - migrate
    - authorability
    - template
    - dx
created_at: 2026-06-16T12:55:29Z
updated_at: 2026-06-16T12:55:29Z
---

extends + modify(suppress:false) + remove blocks (e.g. legal_case extends book) are hard for a style author to reason about. The 'modify: suppress: false' idiom un-suppresses a parent-suppressed component but reads opaquely.

Explore clearer serialization of extends diffs (e.g. explicit add/show vs the suppress toggle). By-design today; lower priority.

Authorability follow-up from ACME review (PR #932). This PR's diff-encoder area.
