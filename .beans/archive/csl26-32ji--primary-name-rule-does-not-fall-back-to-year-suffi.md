---
# csl26-32ji
title: primary-name rule does not fall back to year-suffix when primary names are identical
status: scrapped
type: bug
priority: normal
created_at: 2026-06-02T18:51:59Z
updated_at: 2026-06-02T18:52:18Z
---

When `givenname-disambiguation-rule: primary-name` is active and expanding the first author's given name does not resolve a collision (because both works share an identical primary author), the engine does not fall back to year-suffix as the next cascade step. The two ambiguous citations render identically instead of receiving year suffixes.

Discovered during test soundness review of csl26-lvib (by-cite givenname expansion, PR #868). The `primary_name_givenname_expansion_expands_first_author_only` test pins this broken output: both ASTHMA-A and ASTHMA-B render as `A Asthma, Bronchitis, et al., (1990)` with no suffix.

Expected: `A Asthma, Bronchitis, et al., (1990a)` and `A Asthma, Bronchitis, et al., (1990b)`.

Spec ref: docs/specs/DISAMBIGUATION.md §2 — givenname expansion is one step in the cascade; year-suffix must be applied when the expansion step does not uniquely identify each work.
