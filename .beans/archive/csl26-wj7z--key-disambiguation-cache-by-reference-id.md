---
# csl26-wj7z
title: Key disambiguation cache by reference id
status: completed
type: task
priority: normal
tags:
    - cache
    - sorting
created_at: 2026-07-04T02:42:26Z
updated_at: 2026-07-05T22:53:30Z
parent: csl26-8m2p
---

Disambiguator::reference_cache_key uses the pointer address of &Reference and reference_data expects a hit. Correct only because cache build and lookups happen within one calculate_hints call over an unmoved IndexMap. Key by reference id (index fallback for id-less refs) and drop the expect_used exception. docs/architecture/audits/2026-07-03_CITUM_ENGINE_REVIEW.md finding 7.

## Summary of Changes\n\nReplaced pointer-address disambiguation cache keys with stable per-run keys that use reference ids when available and deterministic index fallback for id-less references. Added focused cache-key coverage and kept anonymous reference grouping deterministic without pointer identity.
