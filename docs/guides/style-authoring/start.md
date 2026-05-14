---
title: Start Writing Citum Styles
nav: Start
description: Learn the mental model for Citum styles and why the YAML format is different from CSL 1.0.
features:
  - style-authoring
---

## [compare_arrows] How Citum Differs

Citum replaces procedural CSL XML with declarative YAML. A style describes the
formatting contract directly: metadata, processing options, citation templates,
bibliography templates, and explicit variants for types that need different
structure.

| Aspect | CSL 1.0 | Citum |
|---|---|---|
| Format | Procedural XML markup | Declarative YAML |
| Logic | `choose` / `if` / `else` trees | type variants and scoped options |
| Name formatting | Inline attributes | global presets and component options |
| Dates | structured year/month/day object | EDTF strings |
| Reuse | parent aliases | explicit `extends` plus scoped overrides |
| Validation | external conventions | schema-backed editor and CLI validation |

> [!TIP]
> **Explicit over hidden processor behavior**
> If journals, books, legal cases, or web pages need different output, put that
> distinction in the style. The engine should not silently guess style intent.

## [rocket_launch] First workflow

1. Pick the closest existing style or preset-backed core style.
2. Write `info`, especially `title`, `id`, and `default-locale`.
3. Set `options.processing` to `author-date`, `numeric`, `note`, or `label`.
4. Add the citation and bibliography templates.
5. Run `citum style validate path/to/style.yaml`.
6. Compare rendered output against CSL, biblatex, or publisher examples.

```yaml
# yaml-language-server: $schema=https://citum.github.io/citum-core/schemas/style.json

info:
  title: "My Style"
  id: "my-style"
  default-locale: "en-US"

options:
  processing: author-date

citation:
  template:
    - contributor: author
    - date: issued

bibliography:
  template:
    - contributor: author
    - date: issued
    - title: primary
```

## [route] Where to go next

- Use **Style Anatomy** when starting a new file.
- Use **Templates** when deciding what a citation or bibliography renders.
- Use **Options** when changing punctuation, names, dates, sorting, or groups.
- Use **Locales** when terms, roles, locators, or grammatical gender matter.
- Use **Inheritance and Registries** when wrapping an existing style.
- Use **Validation** before publishing or comparing fidelity.
