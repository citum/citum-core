---
# csl26-al0f
title: Per-script bibliography partitioning
status: todo
type: feature
priority: high
created_at: 2026-05-01T11:32:07Z
updated_at: 2026-05-01T12:02:48Z
---

Allow multilingual bibliographies to be grouped or sorted per script/language (e.g. Latin names in one section, Arabic in another) rather than interleaved by a single global collator. Currently explicitly out of scope in the Unicode sorting spec — deferred by design. Implement once the single-collator pass is proven stable. Related: UNICODE_BIBLIOGRAPHY_SORTING.md Scope section.

A single global collator produces consistent order, but many users expect separate sections or language/script-first grouping in multilingual lists. This is a common product-level response to the fact that no universal multilingual order feels natural to every reader — it is not a collation bug, but users will report it as one. There is documented user demand for this in citation software. Example: https://forums.zotero.org/discussion/comment/280559/#Comment_280559 — a user with a mixed Latin/CJK bibliography explains the expected grouping behaviour.

The mixed-script bibliography integration test added in csl26-dnz7 (sort_oracle::test_mixed_script_bibliography_sort_stability) validates the single-collator foundation this builds on. This feature likely requires a new schema option (see csl26-xz2t) to let styles request partitioned output.
