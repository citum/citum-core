---
# csl26-wj7z
title: Key disambiguation cache by reference id
status: todo
type: task
tags:
    - cache
    - sorting
parent: csl26-8m2p
created_at: 2026-07-04T02:42:26Z
updated_at: 2026-07-04T17:49:02Z
---

Disambiguator::reference_cache_key uses the pointer address of &Reference and reference_data expects a hit. Correct only because cache build and lookups happen within one calculate_hints call over an unmoved IndexMap. Key by reference id (index fallback for id-less refs) and drop the expect_used exception. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md finding 7.
