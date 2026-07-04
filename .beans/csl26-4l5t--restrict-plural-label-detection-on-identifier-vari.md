---
# csl26-4l5t
title: Restrict plural-label detection on identifier variables
status: todo
type: task
created_at: 2026-07-04T17:12:41Z
updated_at: 2026-07-04T17:12:41Z
---

check_plural treats any -, en-dash, comma, or ampersand as plural and number_var_to_locator_type maps docket/patent/standard/report numbers to LocatorType::Number, so docket-number "19-1392" with a label renders "nos. 19-1392". Restrict range detection to numeric-ish values or exempt identifier-like variables. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 18.
