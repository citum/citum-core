---
# csl26-24pu
title: Locale-specific template layouts for multilingual bibliography
status: completed
type: feature
priority: low
created_at: 2026-02-25T12:21:02Z
updated_at: 2026-03-08T00:00:00Z
---

## Background

PRIOR_ART.md proposes `bibliography.locales[].template` as the Citum pattern for locale-specific bibliography layouts. This allows a style to render the same reference differently depending on the locale — for example, Japanese/CJK mixed-language bibliographies where the component order and punctuation differ from the Latin-script default.

This is implemented on `main`. The schema supports `bibliography.locales[]`,
the processor selects the bibliography template branch from the effective item
language, and localized branches can replace the full entry structure rather
than only locale terms or date formatting.

## Motivation

Real use case: a style that renders Latin-script references as `Author. Title. Publisher, Year.` but CJK-script references as `著者. 出版社, 年. タイトル.` (different component order). Without locale-specific templates, the style cannot correctly handle both in the same bibliography.

CSL-M (the multilingual CSL fork) addresses this with `cs:layout` locale conditions. The Citum approach is cleaner: declare alternate templates per locale directly in the style YAML.

## Implemented shape

```yaml
bibliography:
  template:
    - contributor: author
    - title: primary
  locales:
    - locale: [ja, zh, ko]
      template:
        - contributor: author
        - variable: publisher
        - date: issued
        - title: primary
    - default: true
      template:
        - contributor: author
        - title: primary
```

## Completion proof

- Experimental proof style:
  `styles/experimental/locale-specific-bibliography-layouts.yaml`
- Integration coverage:
  `crates/citum-engine/tests/multilingual.rs`
- Fixture reused:
  `tests/fixtures/multilingual/multilingual-cjk.json`

The proof covers both branches:

- CJK items select the localized bibliography layout
- items without a matching locale fall back to the default layout
- output differences demonstrate full-entry structural switching, not just term localization

## References

- PRIOR_ART.md (locale-specific template proposal)
- MULTILINGUAL.md
- ARCHITECTURAL_SOUNDNESS_2026-02-25.md (gap inventory)
