---
# csl26-24pu
title: Locale-specific template layouts for multilingual bibliography
status: todo
type: feature
priority: low
created_at: 2026-02-25T12:21:02Z
updated_at: 2026-02-25T12:21:02Z
---

## Background

PRIOR_ART.md proposes `bibliography.locales[].template` as the CSLN pattern for locale-specific bibliography layouts. This allows a style to render the same reference differently depending on the locale — for example, Japanese/CJK mixed-language bibliographies where the component order and punctuation differ from the Latin-script default.

This is not yet implemented. Currently a style has one template per reference type; locale overrides apply only to terms and date formats, not to template structure.

## Motivation

Real use case: a style that renders Latin-script references as `Author. Title. Publisher, Year.` but CJK-script references as `著者. 出版社, 年. タイトル.` (different component order). Without locale-specific templates, the style cannot correctly handle both in the same bibliography.

CSL-M (the multilingual CSL fork) addresses this with `cs:layout` locale conditions. The CSLN approach is cleaner: declare alternate templates per locale directly in the style YAML.

## Design sketch (from PRIOR_ART.md)

```yaml
bibliography:
  template:
    - type: article-journal
      components: [...]   # default (Latin-script)
  locales:
    ja:
      template:
        - type: article-journal
          components: [...]   # Japanese layout
```

## Scope

- Schema: add `locales` map to `Bibliography` struct, keyed by BCP 47 language tag
- Processor: detect reference locale from `InputReference.language`, fall back to style default
- Test: at least one style with a Japanese/CJK template variant

## References

- PRIOR_ART.md (locale-specific template proposal)
- MULTILINGUAL.md
- ARCHITECTURAL_SOUNDNESS_2026-02-25.md (gap inventory)
