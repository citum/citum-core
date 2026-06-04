# Interactive API: Per-Document Style Overrides

Status: Active

Bean: csl26-cq35

## Overview

The `format_document` and `open_session` APIs accept an optional `style_overrides`
field containing a partial style (YAML or JSON string). The overlay is merged over
the resolved base style before processing, scoped to that request or session only.
The base style is never mutated.

## Motivation

Word-processor hosts (e.g. citum-office) maintain a single shared style per
project (e.g. APA 7th) but must honour per-document preferences — a different
contributor connector (`&` instead of "and"), tighter et-al thresholds, alternate
particle handling — without editing or duplicating the shared style.

## Field

```json
{
  "style": { "kind": "id", "value": "apa" },
  "style_overrides": "options:\n  contributors:\n    and: symbol\n",
  "refs": …,
  "citations": …
}
```

`style_overrides` accepts any subset of the Citum style YAML schema. JSON is also
accepted (YAML is a JSON superset). Only the keys present in the overlay are
applied; absent keys leave the base style unchanged.

## Merge semantics

The overlay is applied using the same null-aware, typed-merge logic as `extends`
inheritance (`Style::apply_overlay` → `merge_style_overlay`):

- **Options**: granular field-by-field merge via `Config::merge`. Supplying
  `options.contributors.and: symbol` changes only that one sub-key; all other
  contributor settings are inherited from the base style.
- **Citation / bibliography specs**: deep-merged via typed `merge_citation_spec` /
  `merge_bibliography_spec`.
- **Templates**: per-key map merge (overlay key wins).
- **Null clears**: an explicit YAML `~` (null) value for an `Option` field clears
  the inherited value.

## Scope

- `format_document`: overlay applies to that single request.
- `open_session`: overlay is baked into the session style at construction; all
  subsequent mutations within the session use the overridden style.

## Overrideable fields

Any field in the style YAML schema is overrideable, including but not limited to:

- `options.contributors` — `and`, `et-al-min`, `et-al-use-first`, `name-form`,
  `demote-non-dropping-particle`, `display-as-sort`, …
- `options.dates` — date format presets/config
- `options.locators` — locator label config
- `options.titles` — title case/rendering config
- `options.processing` — processing mode (author-date, numeric, …)
- `options.localize` — scope
- `citation.*` — citation template, wrap, collapse, sort
- `bibliography.*` — bibliography template, sort, partition

## Out of scope (future)

- Per-citation overrides (item-level).
- Locale term overrides (tracked separately; see locale gap).
