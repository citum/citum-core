---
# csl26-rqcp
title: 'Processor defect: orphan suffix emits when terminal template component absent'
status: todo
type: bug
priority: high
created_at: 2026-03-22T20:21:02Z
updated_at: 2026-03-22T20:21:02Z
---

## Summary

When the last rendered component in a bibliography template is absent for a
given entry (e.g., no `pages` field), the `suffix` of the preceding component
(e.g., `, `) still emits, producing a trailing delimiter with nothing after it.

## Reproduction

Style: `styles/acm-sig-proceedings.yaml` (or any numeric style with a
pages component carrying a `, ` suffix).

Entry: any reference type where `pages` is not set (e.g., a book without
page range).

Template fragment:
```yaml
- date: issued
  suffix: ", "
- number: pages
```

Expected: `2024` (date only, no trailing comma)
Actual: `2024, ` (orphan suffix emitted)

## Root Cause

The template renderer evaluates and emits each component's suffix independently
of whether the next component will render. A suffix should be suppressed when
the component it precedes produces no output.

## Impact

Affects all numeric styles with comma-delimited bibliography templates where
page numbers are optional. Produces stray trailing punctuation visible in
rendered output.

## Fix Direction

In the template renderer's output assembly: defer suffix emission until the
following component is known to produce output, or implement a
"suppress-if-next-empty" pass over rendered segments before joining.

Found during style-evolve eval run — ACM migration (2026-03-22).

