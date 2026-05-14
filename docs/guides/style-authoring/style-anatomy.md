---
title: Style File Anatomy
nav: Style Anatomy
description: The required top-level pieces of a Citum style file.
features:
  - style-authoring
---

## [folder_special] Top-level structure

Every Citum style has four practical layers:

- `info`: human metadata and compatibility declarations
- `options`: processing mode and shared defaults
- `citation`: citation rendering rules
- `bibliography`: bibliography rendering rules

```yaml
info:
  title: "APA 7th Edition"
  id: "apa-7th"
  default-locale: "en-US"

options:
  processing: author-date
  contributors: apa

citation:
  template:
    - contributor: author
    - date: issued
      form: year

bibliography:
  template:
    - contributor: author
    - date: issued
    - title: primary
```

## [badge] Info fields

Use `info` for identity and compatibility, not formatting.

| Field | Purpose |
|---|---|
| `title` | human-readable style name |
| `id` | stable kebab-case style identifier |
| `description` | short catalog or search description |
| `default-locale` | BCP 47 language tag |
| `fields` | discipline categories |
| `citum-version` | minimum Citum engine requirement |

```yaml
info:
  title: "Hypothetical Journal Style"
  id: "hypothetical-journal"
  default-locale: "en-US"
  citum-version: ">=0.49.0"
```

## [schema] Editor validation

Add the schema comment at the top of hand-authored styles. Editors that support
YAML Language Server use it for autocomplete and inline validation.

```yaml
# yaml-language-server: $schema=https://citum.github.io/citum-core/schemas/style.json
```
