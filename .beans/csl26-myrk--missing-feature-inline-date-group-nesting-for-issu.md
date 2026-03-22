---
# csl26-myrk
title: 'Missing feature: inline date group nesting for issue + year-month in article-journal'
status: todo
type: feature
priority: normal
created_at: 2026-03-22T20:21:06Z
updated_at: 2026-03-22T20:21:06Z
---

## Summary

Citum cannot express a space-delimited sub-group (`issue (Mon Year)`) nested
inside a comma-delimited template sequence. ACM and similar numeric styles
render the volume detail block as:

```
5(3), Jan 2024, pp. 1–12
```

where `3` (issue) and `Jan 2024` (date) are space-delimited and collectively
comma-delimited from the volume number. This pattern requires grouping two
components with a different delimiter than their outer sequence — currently
not expressible in the Citum template schema.

## Current workaround

Place issue and date as sequential components in the comma-delimited template,
producing `5, 3, Jan 2024, pp. 1–12` — functional but not spec-correct.

## Affected styles

ACM SIG Proceedings (`acm-sig-proceedings`), ACM variants, and any numeric
style following the `volume(issue), Mon Year` journal citation convention.

## Design options

1. **`items` with delimiter override** — extend the existing `items` grouping
   construct to accept a `delimiter` field, allowing a sub-list with its own
   separator nested inside the outer template.
2. **Dedicated `group` component** — a first-class `group:` template component
   with child components and a delimiter, analogous to CSL's `<group>`.

Option 1 is lower-lift if `items` already supports mixed content; option 2 is
more general.

## Impact

Affects rendering fidelity for any style using `volume(issue), Mon Year`
article-journal format. Currently produces correct volume/issue numbers but
incorrect delimiter structure around the date.

Found during style-evolve eval run — ACM migration (2026-03-22).

