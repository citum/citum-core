# CSL to Citum Processing Migration

This reference records how `citum-migrate` classifies CSL styles into Citum
`Processing` modes and how disambiguation defaults fold into named presets.

## Classification

Migration classifies processing from CSL structure before template synthesis:

| CSL signal | Citum processing |
| --- | --- |
| `style class="note"` | `note` |
| In-text citation layout renders `citation-number` | `numeric` |
| In-text citation layout renders an author-date signal directly or through a macro | `author-date` or an author-date variant |
| Label-style configuration | `label` |

Template synthesis does not infer disambiguation. Migration reads CSL
declarative attributes, applies the processing-class defaults below, and then
folds the resulting configuration to the closest named preset.

## Class Defaults

| Processing | Sort | Group | Disambiguation |
| --- | --- | --- | --- |
| `author-date` | `author-date-title` | `author`, `year` | `year_suffix: true`, `names: false`, `add_givenname: false` |
| `numeric` | none | none | none |
| `note` | `author-title-date` | none | `year_suffix: false`, `names: true`, `add_givenname: false` |
| `label` | `author-date-title` | none | `year_suffix: true`, `names: false`, `add_givenname: false` |

The author-date default intentionally differs from CSL's raw omitted-attribute
default. Citum treats same-author/same-year year suffixing as the useful
author-date default, while name-list and given-name expansion remain explicit.

## Author-Date Variants

Author-date presets share the same sort and grouping defaults. They differ only
in disambiguation strategy:

| Preset | Year suffix | Name-list expansion | Given-name expansion |
| --- | --- | --- | --- |
| `author-date` | true | false | false |
| `author-date-givenname` | true | false | true |
| `author-date-names` | true | true | false |
| `author-date-full` | true | true | true |

`author-date-full` preserves the previous full-stack behavior under an explicit
name. `author-date` now means year-suffix-only B1 behavior.

## Folding Rules

`citum-migrate` starts from the processing-class default, applies only explicit
CSL disambiguation attributes, and compares the result with named presets:

- A bare author-date CSL style folds to `processing: author-date`.
- Explicit `disambiguate-add-givenname="true"` folds to
  `processing: author-date-givenname`.
- Explicit `disambiguate-add-names="true"` folds to
  `processing: author-date-names`.
- Explicit `disambiguate-add-names="true"` plus
  `disambiguate-add-givenname="true"` folds to
  `processing: author-date-full`.
- Explicit `disambiguate-add-year-suffix="false"` remains custom because it
  contradicts the author-date class default.

`givenname-disambiguation-rule` defaults to `by-cite` when omitted. An explicit
non-default rule is preserved in custom processing unless the full configuration
still exactly matches a named preset.

## Custom as Delta

When folding cannot reach a named preset, migration emits custom processing as
a *delta* on the disambiguation-nearest preset rather than a fully materialized
block. The `base:` field names the preset and only fields that diverge from its
configuration are written:

```yaml
processing:
  base: author-date
  sort:
    template:
      - key: title
```

Resolution overlays present fields onto the base preset's configuration
**wholesale** — a present `sort`, `group`, or `disambiguate` replaces the
base's value entirely; absent fields inherit from the base. There is no
sub-field merge: overriding one disambiguation flag requires restating the
whole `disambiguate` block.

A custom block with a `base` also belongs to the base's processing family:
`regime_family()` and `is_author_date_family()` delegate to the base, and the
default bibliography sort follows the base when no explicit `sort` overrides
it. A `base:` with zero overrides behaves identically to the bare preset.
Base-less custom blocks keep their previous fully-explicit semantics.
