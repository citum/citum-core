---
# csl26-c361
title: Implement or reject NumberForm ordinal/roman
status: todo
type: task
tags:
    - numbers
    - localization
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

TemplateNumber.form (numeric|ordinal|roman) is documented schema surface but TemplateNumber::values never reads it — 'form: ordinal' on edition renders 2 instead of 2nd with no warning; gender agreement similarly limited to label terms. Implement ordinal/roman rendering via locale ordinal suffixes, or reject the option loudly at style load. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 10.
