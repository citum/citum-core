---
# csl26-myrk
title: 'Template V2: inline journal detail grouping for issue + year-month'
status: completed
type: feature
priority: normal
created_at: 2026-03-22T20:21:06Z
updated_at: 2026-03-25T21:30:00Z
---

## Summary

Citum now has the Template V2 surface needed for this work: `group` is the
canonical nesting construct, `items` is legacy naming only, and template-level
component overrides are already gone from `main`. The remaining task is to
emit the correct nested Template V2 shape for article-journal bibliography
detail blocks that structurally match:

- `volume`
- `issue`
- adjacent `issued` date rendered as `year-month`
- within a comma-delimited journal detail sequence

The target Template V2 composition is a nested `group` that preserves
`volume, issue (Mon Year), pages` without relying on flattened sibling
punctuation workarounds.

Spec: `docs/specs/INLINE_JOURNAL_DETAIL_GROUPING.md`

## Current workaround

Touched styles such as `acm-sig-proceedings` currently flatten the detail block
as top-level siblings, which produces the wrong grouping and keeps the stale
pre-Template-V2 representation in style YAML.

## Affected styles

ACM SIG Proceedings (`acm-sig-proceedings`), ACM variants, and any numeric
style following the `volume, issue (Mon Year), pages` journal bibliography
convention.

## Implementation target

- Update migration grouping so adjacent `volume` + `issue` + `issued(year-month)`
  patterns compile to nested Template V2 `group` composition.
- Keep article-journal structure in `type-variants`; do not add any new schema
  fields.
- Normalize touched YAML to `group:` naming.

Found during style-evolve eval run for ACM migration (2026-03-22); reframed on
2026-03-25 after Template V2 landed on `main`.
