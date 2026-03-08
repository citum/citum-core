---
# csl26-4vyv
title: Fix numeric-comp SQI type-coverage accounting in report-core
status: todo
type: bug
priority: normal
created_at: 2026-03-08T00:34:45Z
updated_at: 2026-03-08T00:34:45Z
---

`numeric-comp` is now materially healthier, but the SQI report still appears to
undercount or misclassify its type coverage in `scripts/report-core.js`. The
quality score should reflect the style's actual fallback and type-template
coverage, not legacy assumptions from simpler numeric styles.

Trace how report-core computes type coverage for compound/numeric chemistry
styles, especially where grouped entries, biblatex-backed authorities, or
special-case templates bypass the normal accounting path. Fix the metric so the
reported SQI matches the style definition that ships in `styles/`, then verify
that the resulting change is explainable in `docs/compat.html`.
