<!--
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
-->

# Name Formatting Architecture

## Overview

The Citum name formatting system is a multi-axis design that controls how contributor names (authors, editors, translators) are rendered across different citation contexts. The key innovation is the decomposition of what was previously monolithic configuration into orthogonal, independently controllable axes:

- **Display context**: citation vs. bibliography
- **Citation mode**: integral (narrative) vs. non-integral (parenthetical)
- **Position order**: first mention vs. subsequent mention
- **Name form**: full given names vs. family only vs. initials
- **Name order**: given-first vs. family-first
- **Conjunction**: text ("and") vs. symbol ("&") vs. none

## Formatting Axes Matrix

| Axis | Context | Mode | Position | Form | Order | Conjunction | Already Modeled? |
|------|---------|------|----------|------|-------|-------------|-----------------|
| Display context | Citation | — | — | — | — | — | ✓ RenderContext |
| Display context | Bibliography | — | — | — | — | — | ✓ RenderContext |
| Citation mode | — | Integral | — | — | — | — | ✓ CitationMode |
| Citation mode | — | Non-integral | — | — | — | — | ✓ CitationMode |
| Position order | Citation | — | First | — | — | — | ✓ ProcHints |
| Position order | Citation | — | Subsequent | — | — | — | ✓ ProcHints |
| Name form | — | — | — | Full | — | — | ✓ NameForm::Full |
| Name form | — | — | — | Family only | — | — | ✓ NameForm::FamilyOnly |
| Name form | — | — | — | Initials | — | — | ✓ NameForm::Initials |
| Name order | — | — | — | — | Given-first | — | ✓ NameOrder |
| Name order | — | — | — | — | Family-first | — | ✓ NameOrder |
| Conjunction | — | — | — | — | — | Text | ✓ AndOptions::Text |
| Conjunction | — | — | — | — | — | Symbol | ✓ AndOptions::Symbol |
| Conjunction | — | — | — | — | — | None | ✓ AndOptions::None |

## NameForm Enum

The `NameForm` enum in `citum_schema::options::contributors` controls how the given-name component of a contributor name is rendered:

```rust
pub enum NameForm {
    /// Render full given names: "John D. Smith"
    #[default]
    Full,

    /// Render family name only, suppressing given names: "Smith"
    /// Used for subsequent mentions in Chicago/Turabian note styles
    FamilyOnly,

    /// Render initialized given names using `initialize_with` separator
    /// If `initialize_with` is None, defaults to ". " (e.g., "J. Smith")
    /// Empty string gives compact initials: "JD Smith"
    Initials,
}
```

### Name Form Examples

For a contributor named "John David Smith":

| Form | Example | Use Case |
|------|---------|----------|
| `Full` | "John David Smith" | First mention in citations, all bibliography entries |
| `FamilyOnly` | "Smith" | Subsequent mentions in note styles (Chicago Turabian) |
| `Initials` with `". "` | "J. D. Smith" | Compressed citations, author-date abbreviated second+ mentions |
| `Initials` with `" "` | "J D Smith" | Specific style variant |
| `Initials` with `""` | "JD Smith" | Compact numeric citations |

## Three-Layer Resolution

The effective `NameForm` is resolved through a three-layer configuration hierarchy:

1. **Global level** (`options.contributors.name_form`): Default for all contexts
2. **Scope level** (`citation.options.contributors.name_form`, `bibliography.options.contributors.name_form`): Per-context override
3. **Position/Mode level** (`citation.subsequent.options.contributors.name_form`, `citation.integral.options.contributors.name_form`): Per-position or per-mode override

**Resolution algorithm**:
```
if position-specific name_form exists:
    use position-specific name_form
else if context-specific name_form exists:
    use context-specific name_form
else if global name_form exists:
    use global name_form
else if initialize_with is set:
    treat as NameForm::Initials (backward compat)
else:
    use NameForm::Full (default)
```

### Citum Configuration Chain

In YAML, the chain looks like:

```yaml
options:
  contributors:
    name-form: full            # Layer 1: global default
    initialize-with: ". "      # Formatting detail for Initials

citation:
  subsequent:
    options:
      contributors:
        name-form: family-only # Layer 3: override for subsequent mentions
    template: [ ... ]

  integral:
    options:
      contributors:
        and: text              # Example: override and-style for integral mode
    template: [ ... ]
```

## Initialization Rules

The `initialize_with` and `initialize_with_hyphen` fields are **formatting details** for the `NameForm::Initials` variant, not a switch themselves:

- `initialize_with: ". "` → "J. Smith"
- `initialize_with: " "` → "J Smith"
- `initialize_with: ""` → "JSmith"
- `initialize_with_hyphen: false` → suppress hyphen in compound initials (e.g., "J.-P." becomes "J. P.")

**Backward compatibility**: If `name_form` is `None` and `initialize_with` is `Some`, the engine treats it as `NameForm::Initials` for existing styles.

## Use Cases

### APA 7th Edition

APA uses different conjunctions in integral vs. non-integral citations:

```yaml
options:
  contributors:
    and: symbol  # Default: & in non-integral (parenthetical)

citation:
  integral:
    options:
      contributors:
        and: text  # Override: "and" in integral (narrative)
    template: [ ... ]
```

### Chicago Manual of Style (Notes & Bibliography)

Chicago uses full names on first mention, family-only on subsequent mentions within the same footnote series:

```yaml
citation:
  template:
    - contributor: author
      form: long  # Full name rendering

  subsequent:
    options:
      contributors:
        name-form: family-only  # Override for subsequent cites
    template:
      - contributor: author
        form: short  # Abbreviated form (but still renders family name only)
```

### Scientific Author-Date Styles

Numeric and author-date styles often initialize given names to compress citations:

```yaml
options:
  contributors:
    name-form: initials
    initialize-with: "."  # "J. Smith" in citations

bibliography:
  options:
    contributors:
      name-form: full     # "John Smith" in bibliography
```

## ModeDependent Removal

Prior to this redesign, APA and similar styles used `AndOptions::ModeDependent` to express context-sensitive behavior:

```yaml
# OLD (CSL 1.0):
and:
  mode-dependent:
    integral: text
    non-integral: symbol
```

This was a **category error**: conjunction is orthogonal to citation mode. The fix is explicit per-scope configuration:

```yaml
# NEW (Citum):
contributors:
  and: symbol  # Default for all contexts

citation:
  integral:
    options:
      contributors:
        and: text  # Override for integral mode
```

**Benefits**:
- Clearer semantics: per-scope options are declarative, not imperative
- Composability: independent axes (name form, conjunction, order) can be mixed freely
- Extensibility: future additions (e.g., per-scope `initialize_with`) follow the same pattern

## biblatex Parallels

Citum's design aligns with biblatex name formatting configuration:

| Citum | biblatex | Meaning |
|-------|----------|---------|
| `NameForm::Full` | `nameformat = given-family` | Full given + family |
| `NameForm::FamilyOnly` | `nameformat = family` | Family name only |
| `NameForm::Initials` | `nameformat = given-family, giveninits` | Initials + family |
| `AndOptions::Text` | `and = and` | Locale-aware conjunction |
| `AndOptions::Symbol` | `and = ampersand` | "&" symbol |

Citum does **not** model biblatex's author-editor-translator disambiguation directly; that is handled via separate `ContributorRole` attributes.

## Future Extensions

### Per-Scope `initialize_with` Override

Future support for style-specific initialization separators:

```yaml
citation:
  non-integral:
    options:
      contributors:
        name-form: initials
        initialize-with: ". "  # Default ". "

  integral:
    options:
      contributors:
        name-form: initials
        initialize-with: " "   # Space separator for integral mode
```

### Locale-Aware Punctuation

Coordination with locale (language) settings for comma/period placement in initials across different scripts.

## Testing & Validation

All NameForm variants are exercised in:
- `crates/citum-engine/src/values/tests.rs`: unit tests for `format_single_name`
- `styles/apa-7th.yaml`, `styles/chicago-notes.yaml`: real-world integration tests
- Oracle suite: comparing against CSL 1.0 rendering for compatibility

## References

- **CSL 1.0**: [Name Fields](https://citeproc-js.readthedocs.io/en/latest/csl-json/types.html#name-fields)
- **CSL-M**: [Names](https://csl-m.readthedocs.io/en/latest/csl-m-intro.html)
- **biblatex**: [Author Name Formatting](https://ctan.org/pkg/biblatex)
- **Chicago Manual of Style 18th**: [Notes and Bibliography](https://www.chicagomanualofstyle.org/tools_citationguide_notes.html)
- **APA Style 7th**: [In-text Citations](https://apastyle.apa.org/style-grammar-guidelines/citations/basic-principles)
