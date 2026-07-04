---
# csl26-2ubj
title: Unify page/locator range formats; fix CMOS rule
status: completed
type: task
priority: normal
tags:
    - numbers
    - rendering
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T18:33:49Z
parent: csl26-8m2p
---

Three defects: format_chicago renders 101-108 as 101–08 where CMOS 17/citeproc-js give 101–8; locator apply_range_format implements Minimal as strip-only-when-already-abbreviated and Chicago falls through to expanded, so the same option formats pages and locators differently; labeled locator segments hardcode Expanded, ignoring config range_format. Share one range-format implementation between number.rs and locator.rs. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 11.

## Summary of Changes

Unified page and locator range formatting on the engine shared page-range formatter. Implemented the newer Chicago page-range abbreviation rules, and added unit coverage for Chicago table examples plus labeled/unlabeled locator range-format precedence.
