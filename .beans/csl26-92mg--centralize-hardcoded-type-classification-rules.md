---
# csl26-92mg
title: Centralize hardcoded type-classification rules
status: todo
type: task
tags:
    - types
    - engine
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

Six inconsistent engine sites hardcode per-type presentation: dataset [Dataset] suffix string literal in get_effective_rendering; parent_short_title gates on ref_type.contains("article"); type_class_matches hardcodes legal/classical lists incl. contains("ancient"); get_title_category_rendering legacy type tables; SimpleVariable::Url DOI synthesis only for dataset; chapter silently matching entry-dictionary type-variants. Centralize into one classification table and replace the [Dataset] literal with a schema option. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 14.
