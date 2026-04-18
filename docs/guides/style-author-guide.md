# Style Author Guide

This guide is for people who write and maintain Citum styles.

## [compare_arrows] How Citum Differs

Citum introduces a modern, declarative approach to citation styling compared to CSL 1.0's procedural XML language. Understanding these differences is essential for writing Citum styles effectively.

| Aspect | CSL 1.0 (XML) | Citum (YAML) |
|---|---|---|
| Format | Procedural XML markup | Declarative YAML |
| Logic | `choose`/`if`/`else` conditionals | Type variants + inheritance |
| Name Formatting | Inline XML attributes | Global presets + options |
| Dates | Object with year/month/day | EDTF string format |
| Inheritance | Parent style aliasing | Presets + type variants |
| Readability | Verbose and nested | Concise and explicit |

> [!TIP]
> **Explicit Over Magic**
> Citum styles are explicitly declarative. Special behavior is expressed in the style itself, not hidden in processor logic. If you need a different layout for journals vs books, you declare it with type variants. This makes styles portable, testable, and understandable without reading source code.

## [folder_special] Style Anatomy

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

## [auto_awesome] Style Presets

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

> [!TIP]
> **Type Variants Replace the Default Template**
> If a reference type matches a `type-variants` entry, that template is used in full instead of the default template. Types not listed fall through to the default.

## [article] Citation Modes

Use different citation templates for narrative vs. parenthetical citations, shortened forms, and special cases like ibid.

### Integral vs Non-Integral Citations

Use `integral:` and `non-integral:` blocks for narrative ("Smith (2020) argued...") vs parenthetical ("...was argued (Smith, 2020)") styles:

```yaml
citation:
  wrap: parentheses       # default wrapping
  template:               # used as fallback if no mode-specific block
    - contributor: author
    - date: issued
  non-integral:           # (Smith, 2020)
    wrap: parentheses
    template:
      - contributor: author
        form: short
      - date: issued
        form: year
  integral:               # Smith (2020)
    delimiter: " "
    template:
      - contributor: author
        form: short
      - date: issued
        form: year
        wrap: parentheses
```

### Note-style: Subsequent and Ibid

Use `subsequent:` for shortened second-and-later citations, and `ibid:` for same-source repetitions:

```yaml
citation:
  template:               # Full first citation
    - contributor: author
      form: long
    - title: primary
      prefix: ", "
    - variable: locator
      prefix: ", "
  subsequent:             # Short form for later citations
    options:
      contributors:
        name-form: family-only
    template:
      - contributor: author
        form: short
      - title: primary
        form: short
      - variable: locator
        prefix: ", "
  ibid:                    # Same source, possibly different locator
    suffix: "Ibid."
    template:
      - variable: locator
        prefix: ", "
```

### Multi-cite and Collapse

For numeric styles, use `multi-cite-delimiter` (default "; ") to separate multiple citations, and `collapse: citation-number` to render ranges like [1–3]:

- `multi-cite-delimiter`: String to separate multiple citations.
- `collapse: citation-number`: Consecutive numbers are collapsed into ranges.

## [share] Options Inheritance

Global options apply to all components, but can be overridden at the citation and bibliography level.

```yaml
options:
  contributors: apa       # Global: family-first, initials, up to 20 names

citation:
  options:
    contributors:
      shorten: { min: 3, use-first: 1 }   # Citation-level override

bibliography:
  options:
    contributors:
      shorten: { min: 20, use-first: 19 }  # Bibliography-level override
```

### Inheritance Chain

1. Component-level options (highest priority)
2. Citation/Bibliography-level options
3. Global options (lowest priority)

## [sort] Bib Sort & Groups

Control bibliography ordering and split it into labeled sections based on reference properties.

### Sort

The `bibliography.sort` field controls ordering. Use a preset string or a custom sort template:

```yaml
bibliography:
  sort: author-date-title   # preset: sort by author, then date, then title
```

**Preset sort values:** `author-date-title`, `author-title-date`.

### Groups

The `bibliography.groups` field splits the bibliography into labeled sections:

```yaml
bibliography:
  groups:
    - id: primary
      heading:
        localized:
          en-US: "Primary Sources"
      selector:
        type: legal-case
    - id: other
      heading:
        localized:
          en-US: "Secondary Sources"
      selector:
        not:
          type: legal-case
```

## [list] Reference Types

References use a `class` (top-level discriminator) and `type` (subtype). In styles, use the `type` value as keys under `type-variants`.

| Class | Types |
|---|---|
| Monograph | `book`, `manual`, `report`, `thesis`, `webpage`, `post`, `interview`, `manuscript`, `document` |
| Collection | `anthology`, `proceedings`, `edited-book`, `edited-volume` |
| Component | `chapter`, `article-journal`, `article-magazine`, `article-newspaper`, `broadcast`, `post` |
| Standalone | `legal-case`, `statute`, `treaty`, `hearing`, `regulation`, `brief`, `patent`, `dataset`, `standard`, `software` |

> [!TIP]
> **Using Type Values in type-variants**
> - Use the `type` value (e.g., `article-journal`, `book`) in `type-variants:`.
> - Special keywords `default` and `all` also work.
> - For components, parents are embedded under the `parent:` key.

## [code] Complete Examples

### Example 1: Minimal Author-Date Style

```yaml
info:
  title: "Simple Author-Date"
  id: "simple-author-date"

options:
  processing: author-date
  contributors: apa

citation:
  template:
    - contributor: author
    - date: issued
      prefix: " "
      wrap: parentheses

bibliography:
  template:
    - contributor: author
    - date: issued
      prefix: " "
    - title: primary
      prefix: " "
      suffix: "."
```

### Example 2: Minimal Numeric Style

```yaml
info:
  title: "Simple Numeric"
  id: "simple-numeric"

options:
  processing: numeric

citation:
  template:
    - number: citation-number
      wrap: brackets

bibliography:
  template:
    - number: citation-number
      suffix: ". "
    - contributor: author
    - title: primary
      prefix: " "
```

## [lightbulb] Workflow & Tips

### Recommended Workflow

1. **Start with a reference style**: Use an existing style as a template.
2. **Write metadata**: Set title, id, and default locale in `info`.
3. **Define global options**: Set mode and contributor/date presets.
4. **Write citation template**: Start with author, date, and title.
5. **Test with oracle**: Compare output against reference implementation.
6. **Add type-variants**: Only for types needing a structurally different template.

### Common Mistakes to Avoid

> [!WARNING]
> **Over-using type-variants**
> Only add `type-variants` for types that need a genuinely different component set. Use presets and a well-designed generic template for the common case.

> [!WARNING]
> **Over-complicated options inheritance**
> Keep global options simple. Override only at citation/bibliography level when truly needed.

> [!WARNING]
> **Mismatching prefix/suffix pairs**
> Always pair opening prefix with closing suffix. For structural wrapping (e.g. parentheses), use the `wrap` option instead.
