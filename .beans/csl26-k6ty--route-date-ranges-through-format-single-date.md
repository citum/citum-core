---
# csl26-k6ty
title: Route date ranges through format_single_date
status: todo
type: task
tags:
    - dates
    - rendering
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

format_range_start is a ~100-line near-duplicate of format_single_date without locale pattern resolution, and extract_range_end hardcodes English month-day-year assembly with long month names. Route both range endpoints through format_single_date (year-suppressed variant for same-year ends) and delete the duplicate. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 5.
