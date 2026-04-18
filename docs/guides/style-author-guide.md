# Style Author Guide

This guide is for people who write and maintain Citum styles.

## [compare_arrows] How Citum Differs from CSL 1.0

Citum introduces a modern, declarative approach to citation styling compared to CSL 1.0's procedural XML language. Understanding these differences is essential for writing Citum styles effectively.

| Aspect | CSL 1.0 (XML) | Citum (YAML) |
|---|---|---|
| Format | Procedural XML markup | Declarative YAML |
| Logic | choose/if/else conditionals | Type variants + inheritance |
| Name Formatting | Inline XML attributes | Global presets + options |
| Dates | Object with year/month/day | EDTF string format |
| Inheritance | Parent style aliasing | Presets + type variants |
| Readability | Verbose and nested | Concise and explicit |

> [!TIP]
> **Explicit Over Magic**
> Citum styles are explicitly declarative. Special behavior is expressed in the style itself, not hidden in processor logic. If you need a different layout for journals vs books, you declare it with type variants. This makes styles portable, testable, and understandable without reading source code.

## [folder_special] Style File Anatomy

Every Citum style file contains four top-level sections: metadata, options, citation template, and bibliography template.

### Minimal Style Skeleton

```yaml
# yaml-language-server: $schema=https://citum.github.io/citum-core/schemas/style.json

info:
  title: "My Style Name"
  id: "my-style"
  description: "Optional short description"
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

> [!TIP]
> **Editor autocomplete and validation**
> The `yaml-language-server` comment in the skeleton above enables full autocomplete and inline validation in editors that support the [YAML Language Server protocol](https://github.com/redhat-developer/yaml-language-server).

### Info Fields

- `title`: Human-readable name (e.g., "American Psychological Association 7th Edition").
- `id`: Unique identifier in kebab-case (e.g., "apa-7th").
- `default-locale`: BCP 47 language tag (e.g., "en-US", "de-DE").
- `fields`: Discipline categories (e.g., anthropology, biology, history).

## [tune] Global Options

Global options control the processing mode and apply defaults to all components in both citation and bibliography templates.

### Processing Modes

| Mode | Description | Example |
|---|---|---|
| `author-date` | Author+year/page citations | (Smith, 2020) |
| `numeric` | Numbered citations | [1] |
| `note` | Footnote-based citations | Smith, "Title," 2020 |
| `label` | Alphabetic or numeric keys | [Kuh62] |

### Contributor Presets

Named presets control name formatting without spelling out every field:

- `apa`: Family-first, "&" symbol, initials with period-space.
- `chicago`: Family-first, "and" text, full names.
- `vancouver`: All family-first, no conjunction, compact initials.
- `ieee`: Given-first, "and" text, initials with period-space.
- `harvard`: All family-first, "and" text, compact initials.
- `springer`: All family-first, no conjunction, compact initials.

### Date Presets

| Preset | Format | Example |
|---|---|---|
| `long` | Full month names, EDTF markers | January 15, 2024 |
| `short` | Abbreviated month names | Jan 15, 2024 |
| `numeric` | Numeric months | 1/15/2024 |
| `iso` | ISO 8601, no EDTF markers | 2024-01-15 |

## [layers] Template Components

### Contributor
Renders author, editor, translator, and other contributors.

```yaml
- contributor: author
  form: long          # long | short | verb | verb-short
  name-order: family-first  # family-first | given-first
```

### Date
Renders date fields using EDTF format.

```yaml
- date: issued
  form: year  # year | year-month | full | month-day | year-month-day
```

### Title
Renders the title of the item.

```yaml
- title: primary
  form: long  # long | short
```

### Number
Renders numeric data: volume, issue, pages, edition, etc.

```yaml
- number: pages
  form: numeric  # numeric | ordinal | roman
```

## [formatting] Rendering Options

Every component can be modified with rendering options that control punctuation, formatting, and text wrapping.

> [!WARNING]
> **Avoid locale-specific strings**
> Do not hardcode text content like `"In: "`, `"Editor"`, or `"pp. "` in `prefix` or `suffix`. Always use the `term` component for localized text. Reserve prefix and suffix for punctuation and spacing.

| Option | Values | Purpose |
|---|---|---|
| `prefix` | `" "`, `", "`, `". "` | Text before the component |
| `suffix` | `".", ",", ": "` | Text after the component |
| `wrap` | `parentheses`, `brackets`, `quotes` | Automatically wrap value |
| `emph` | `true`, `false` | Render in italics |
| `strong` | `true`, `false` | Render in bold |

## [auto_awesome] Style-Level Presets

Reference named, compiled-in styles at the top of your YAML.

```yaml
preset: chicago-notes-18th
```

## [category] Type Variants

When reference types need a different layout, use `type-variants`.

```yaml
bibliography:
  template:
    - contributor: author
  type-variants:
    article-journal:
      - contributor: author
      - title: primary
```

## [psychology] Workflow & Tips

> [!WARNING]
> **Over-complicated options inheritance**
> Keep global options simple. Override only at citation/bibliography level when truly needed.

> [!WARNING]
> **Mismatching prefix/suffix pairs**
> Always pair opening prefix with closing suffix. For structural wrapping (e.g. parentheses), use the `wrap` option instead.
