---
title: Options, Sorting, and Document Inputs
nav: Options
description: Configure shared formatting behavior without duplicating templates.
features:
  - style-authoring
  - abbreviation-map
---

## [tune] Global options

Global options apply across citation and bibliography rendering unless a
section or component overrides them.

```yaml
options:
  processing: author-date
  contributors: apa
  dates: long
```

| Option family | Controls |
|---|---|
| `processing` | author-date, numeric, note, or label behavior |
| `contributors` | name order, initials, conjunctions, shortening |
| `dates` | long, short, numeric, or ISO date rendering |
| `titles` | title casing and title form behavior |

## [formatting] Component rendering options

Use component options for punctuation, wrapping, and emphasis.

```yaml
- title: parent-serial
  prefix: ". "
  emph: true

- number: pages
  prefix: ", "
  label-form: short
```

> [!WARNING]
> **Do not hardcode localized labels**
> Avoid `prefix: "pp. "` or `prefix: "In: "`. Use `term` components or
> label options so locale files own language-specific text.

## [share] Options inheritance

More local options override broader options:

1. component-level options
2. citation or bibliography section options
3. global options

```yaml
options:
  contributors: apa

citation:
  options:
    contributors:
      shorten: { min: 3, use-first: 1 }
```

## [sort] Bibliography sort and groups

```yaml
bibliography:
  sort: author-date-title
  groups:
    - id: primary
      heading:
        localized:
          en-US: "Primary Sources"
      selector:
        type: legal-case
```

## [short_text] Document-level abbreviation maps

Abbreviation maps belong to the document input, not the style. They replace
full rendered strings after value extraction and before output assembly.

```yaml
abbreviation-map:
  Estates Gazette: EG
  "Lloyd's Law Reports": "Lloyd's Rep"
  "World Health Organization": WHO
```
