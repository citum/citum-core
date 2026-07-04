---
# csl26-2ubj
title: Unify page/locator range formats; fix CMOS rule
status: todo
type: task
tags:
    - numbers
    - rendering
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

Three defects: format_chicago renders 101-108 as 101–08 where CMOS 17/citeproc-js give 101–8; locator apply_range_format implements Minimal as strip-only-when-already-abbreviated and Chicago falls through to expanded, so the same option formats pages and locators differently; labeled locator segments hardcode Expanded, ignoring config range_format. Share one range-format implementation between number.rs and locator.rs. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 11.
