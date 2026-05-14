---
title: Templates and Type Variants
nav: Templates
description: Build citation and bibliography output from explicit template components.
features:
  - style-authoring
---

## [layers] Template components

Templates are ordered component lists. Each component names the data it renders
and optional formatting around that data.

```yaml
bibliography:
  template:
    - contributor: author
    - date: issued
      form: year
      wrap: parentheses
      prefix: " "
    - title: primary
      prefix: " "
    - title: parent-serial
      emph: true
      prefix: ". "
```

| Component | Common use |
|---|---|
| `contributor` | author, editor, translator, and other contributor roles |
| `date` | issued, accessed, original-date, and other date fields |
| `title` | primary title, parent title, collection title |
| `number` | volume, issue, pages, edition, citation number |
| `term` | localized labels such as page, volume, or editor |
| `variable` | direct field output such as publisher, DOI, URL |

## [category] Type variants

Use `type-variants` when a reference type needs a different bibliography
structure. Prefer small diff variants when the default template is mostly
right.

```yaml
bibliography:
  template:
    - contributor: author
    - title: primary
    - variable: publisher
      prefix: ". "

  type-variants:
    article-journal:
      modify:
        - match:
            title: primary
          suffix: ". "
      add:
        - after:
            title: primary
          component:
            title: parent-serial
            emph: true
      remove:
        - match:
            variable: publisher
```

Selectors are partial component matches. `{ title: primary }` matches a primary
title component even when that component also has affixes or emphasis.

## [article] Citation modes

Use mode-specific citation blocks when narrative and parenthetical citations
diverge.

```yaml
citation:
  non-integral:
    wrap: parentheses
    template:
      - contributor: author
        form: short
      - date: issued
        form: year
        prefix: ", "
  integral:
    delimiter: " "
    template:
      - contributor: author
        form: short
      - date: issued
        form: year
        wrap: parentheses
```

Note styles can also define `subsequent` and `ibid` templates for repeated
citations.
