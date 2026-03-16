# Citum Engine

The core citation and bibliography processing engine for Citum.

## Document Input Formats

The engine supports two document input formats for the `process_document` pipeline.
These are independent from the **document output format** (HTML, Plain, LaTeX, Typst)
and from **reference field inline markup** (Djot inline in `title`, `annotation`, and
`note` fields via `render_djot_inline`).

### Djot

Full-featured adapter. In addition to citation parsing, the Djot adapter supports:
- YAML frontmatter (bibliography groups, integral-name overrides)
- Manual footnotes (note-style placement into authored `[^label]:` blocks)
- Inline bibliography blocks (`:::bibliography`)
- Djot-to-HTML finalization via `jotdown`

#### Citation Syntax

| Syntax | Description | Example (APA) |
|--------|-------------|---------------|
| `[@key]` | Basic parenthetical citation | (Smith, 2023) |
| `[@key1; @key2]` | Multiple citations | (Smith, 2023; Jones, 2022) |
| `[prefix ; @key1; @key2]` | Global prefix | (see Smith, 2023; Jones, 2022) |
| `[@key1; @key2 ; suffix]` | Global suffix | (Smith, 2023; Jones, 2022 for more) |
| `[prefix ; @key1; @key2 ; suffix]` | Both global affixes | (see Smith, 2023; Jones, 2022 for more) |

**Note on Semicolons**: Global affixes must be separated from cite keys by a semicolon `;`.

#### Narrative (Integral) Citations

Narrative citations are integrated into the text flow using the `+` mode modifier. For numeric styles, these render as **Author [1]**.

| Syntax | Description | Example |
|--------|-------------|---------| 
| `[+@key]` | Explicit narrative | Smith (2023) |

#### Modifiers

Modifiers appear immediately before the `@` symbol.

| Modifier | Type | Description | Syntax | Result |
|----------|------|-------------|--------|--------|
| `-` | Visibility | Suppress Author | `[-@key]` | (2023) |
| `+` | Mode | Integral / Narrative | `[+@key]` | Smith (2023) |
| `!` | Visibility | Hidden (Nocite) | `[!@key]` | *bibliography only* |

#### Locators (Pinpoints)

Locators follow a comma after the citekey.

| Type | Syntax | Result |
|------|--------|--------|
| **Page** | `[@key, 45]` or `[@key, p. 45]` | (Smith, 2023, p. 45) |
| **Chapter** | `[@key, ch. 5]` | (Smith, 2023, ch. 5) |
| **Structured** | `[@key, chapter: 2, page: 10]` | (Smith, 2023, chap. 2, p. 10) |

Bare locator values default to `page`.

#### Complex Examples

- **Explicit narrative**: `[+@smith2023]` → Smith (2023)
- **Mixed visibility**: `[see ; -@smith2023, p. 45; @jones2022]` → (see 2023, p. 45; Jones, 2022)
- **Global affixes**: `[compare ; @kuhn1962; @watson1953 ; for discussion]` → (compare Kuhn, 1962; Watson & Crick, 1953 for discussion)

---

### Markdown (Pandoc-style)

Lightweight adapter for Markdown documents that use Pandoc-style citation markers.
Supports inline prose citations only. Frontmatter, manual footnotes, and inline
bibliography blocks are not supported in this adapter (v1).

HTML output is **not** post-processed by this adapter — the caller receives the
citation-substituted Markdown markup and is responsible for any subsequent
CommonMark-to-HTML conversion.

#### Citation Syntax

| Syntax | Description |
|--------|-------------|
| `[@key]` | Parenthetical citation |
| `[@key1; @key2]` | Multi-cite cluster |
| `[-@key]` | Suppress-author citation |
| `@key` | Textual (narrative/integral) citation |
| `@key [p. 45]` | Textual citation with bracketed locator |
| `[@key, p. 45]` | Parenthetical citation with locator |

The Pandoc citation syntax is syntactically very similar to Djot's — both formats
share `@key` as the citation token and `[...]` for grouping. The Markdown adapter
intentionally supports the common subset; Djot-specific modifiers (`+`, `!`) are
not part of Pandoc syntax and are not supported here.

---

## Reference Field Inline Markup

Inline markup in bibliographic data fields (`title`, `annotation`, `note`) is a
**separate concern** from document input format. Field markup uses Djot inline
syntax via `render_djot_inline`, regardless of which document input format is in use.

See `docs/architecture/DJOT_RICH_TEXT.md` for details.
