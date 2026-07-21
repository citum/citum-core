# citum-engine

`citum-engine` is the Rust citation and bibliography processor for Citum. Use
it when your application already has, or can load, Citum styles, reference data,
and citation occurrences, and needs formatted citations or bibliographies as
plain text, HTML, Djot, LaTeX, Typst, or CommonMark/Markdown.

The engine is intentionally narrower than the full Citum application stack. It
does not manage registries, fetch remote styles, or persist user data. Those
responsibilities belong in adapters such as a CLI, server, editor integration,
or application-specific resolver.

## Installation

```toml
[dependencies]
citum-engine = "0.73"

# Optional, but recommended when loading bibliography files.
citum-io = "0.73"
```

Locale-tailored text casing is available in every build through ICU4X. Default
features also include `icu`, which enables ICU-backed collation for sorting.
Optional features:

| Feature | Purpose |
|---|---|
| `ffi` | Enables the C ABI module. |
| `schema` | Enables JSON Schema generation support for API-facing types. |
| `parallel` | Opts into Rayon bibliography rendering above the internal size threshold; profile before enabling. |

## Which API to Use

Most integrations should start with the document-level API:

- `format_document` resolves a local style from `StyleInput::Path` or
  `StyleInput::Yaml`, and bibliography data from `RefsInput::Path`,
  `RefsInput::Yaml`, or `RefsInput::Json`, then formats ordered citation
  occurrences and a bibliography.
- `format_document_with_style` takes an already resolved `Style` as its first
  argument, plus the same `FormatDocumentRequest` shape. Use this when your
  application has its own style resolver, registry, cache, or embedded style
  bundle.

Use `Processor` directly when you need lower-level control over individual
citations, bibliography rendering, output-format generics, locales, or parsed
document pipelines.

The `Processor::process_document` path is an advanced document parser pipeline
for Djot and a common subset of Pandoc-style Markdown citation syntax. If your
application already parses a document into citation occurrences, prefer
`format_document` or `format_document_with_style`.

## Quick Start

This example uses the document-level API with local Citum YAML files.
`RefsInput::Path` reads and parses the bibliography file at request time;
`RefsInput::Yaml` and `RefsInput::Json` accept inline data when loading from
a path is not convenient.

```rust,no_run
use citum_engine::{
    format_document, CitationOccurrence, CitationOccurrenceItem,
    FormatDocumentRequest, OutputFormatKind, RefsInput, StyleInput,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = FormatDocumentRequest {
        style: StyleInput::Path("styles/embedded/apa-7th.yaml".to_string()),
        locale: None,
        output_format: OutputFormatKind::Html,
        refs: RefsInput::Path("references.yaml".to_string()),
        citations: vec![CitationOccurrence {
            id: "cite-1".to_string(),
            items: vec![CitationOccurrenceItem {
                id: "kuhn1962".to_string(),
                locator: None,
                prefix: None,
                suffix: None,
                integral_name_state: None,
            }],
            mode: None,
            note_number: None,
            suppress_author: None,
            grouped: None,
            prefix: None,
            suffix: None,
        }],
        document_options: None,
    };

    let result = format_document(request)?;

    for citation in result.formatted_citations {
        println!("{}: {}", citation.id, citation.text);
    }

    println!("{}", result.bibliography.content);

    for warning in result.warnings {
        eprintln!("{}: {}", warning.code, warning.message);
    }

    Ok(())
}
```

If your application resolves styles before calling the engine, parse or load the
style into `citum_schema::Style` and call `format_document_with_style(style,
request)` instead. `StyleInput::Id` and `StyleInput::Uri` are accepted in request
types for adapter compatibility, but `format_document` cannot resolve them by
itself. `format_document_with_style` still requires `FormatDocumentRequest.style`
to be populated; callers usually keep the original `StyleInput` there for
serialization or diagnostics, while the function renders with the explicit
`Style` argument.

## Inputs and Outputs

The document-level API expects:

| Input | Type | Notes |
|---|---|---|
| Style | `StyleInput`; optionally a separate resolved `Style` | `format_document` resolves local path or inline YAML values from `request.style`. `format_document_with_style` uses its explicit `Style` argument, but the request still includes a `style` field. |
| References | `RefsInput` | Local bibliography path, inline YAML, inline JSON, or legacy bare JSON map. |
| Citations | `Vec<CitationOccurrence>` | Ordered as they appear in the document so note positions and repeated citations can be processed. |
| Output format | `OutputFormatKind` | `plain`, `html`, `djot`, `latex`, `typst`, or `markdown`. |

The result contains formatted citations, a formatted bibliography, per-entry
bibliography metadata, and structured warnings. Missing references and
forward-compatible unknown values are reported as warnings where rendering can
continue.

## Boundaries

- The engine does not fetch remote styles, access registries, or resolve style
  IDs and URIs. Resolve those before calling `format_document_with_style`, or use
  another Citum adapter that provides a resolver chain.
- `format_document` uses the built-in `en-US` locale. A non-`en-US` locale tag
  currently produces a warning and falls back unless the caller uses lower-level
  APIs with a resolved locale.
- Markdown document parsing supports the common Pandoc-style citation subset
  listed below. It does not implement all Djot document features.
- Inline markup inside bibliographic fields such as `title`, `annotation`, and
  `note` is Djot-based and independent from the document input format.

## Document Syntax

The engine supports two document input formats for the `process_document`
pipeline. These are independent from the document output format and from
reference-field inline markup.

### Djot

The Djot adapter supports citation parsing plus:

- YAML frontmatter for bibliography groups and integral-name overrides
- manual footnotes for note-style placement into authored `[^label]:` blocks
- inline bibliography blocks with `:::bibliography`
- Djot-to-HTML finalization via `jotdown`

#### Citation Syntax

| Syntax | Description | Example (APA) |
|---|---|---|
| `[@key]` | Basic parenthetical citation | `(Smith, 2023)` |
| `[@key1; @key2]` | Multiple citations | `(Smith, 2023; Jones, 2022)` |
| `[prefix ; @key1; @key2]` | Global prefix | `(see Smith, 2023; Jones, 2022)` |
| `[@key1; @key2 ; suffix]` | Global suffix | `(Smith, 2023; Jones, 2022 for more)` |
| `[prefix ; @key1; @key2 ; suffix]` | Both global affixes | `(see Smith, 2023; Jones, 2022 for more)` |

Global affixes must be separated from cite keys by a semicolon (`;`).

#### Narrative Citations

Narrative citations are integrated into the text flow using the `+` mode
modifier. For numeric styles, these render as `Author [1]`.

| Syntax | Description | Example |
|---|---|---|
| `[+@key]` | Explicit narrative | `Smith (2023)` |

#### Modifiers

Modifiers appear immediately before the `@` symbol.

| Modifier | Type | Description | Syntax | Result |
|---|---|---|---|---|
| `-` | Visibility | Suppress author | `[-@key]` | `(2023)` |
| `+` | Mode | Integral / narrative | `[+@key]` | `Smith (2023)` |
| `!` | Visibility | Hidden / nocite | `[!@key]` | Bibliography only |

#### Locators

Locators follow a comma after the cite key. Bare locator values default to
`page`.

| Type | Syntax | Result |
|---|---|---|
| Page | `[@key, 45]` or `[@key, p. 45]` | `(Smith, 2023, p. 45)` |
| Chapter | `[@key, ch. 5]` | `(Smith, 2023, ch. 5)` |
| Structured | `[@key, chapter: 2, page: 10]` | `(Smith, 2023, chap. 2, p. 10)` |

#### Complex Examples

- `[+@smith2023]` renders as `Smith (2023)`.
- `[see ; -@smith2023, p. 45; @jones2022]` renders as
  `(see 2023, p. 45; Jones, 2022)`.
- `[compare ; @kuhn1962; @watson1953 ; for discussion]` renders as
  `(compare Kuhn, 1962; Watson & Crick, 1953 for discussion)`.

### Markdown (Pandoc-style)

The Markdown adapter supports inline prose citations using a common subset of
Pandoc-style citation markers. Frontmatter, manual footnotes, and inline
bibliography blocks are not supported by this adapter.

HTML output is not post-processed by the Markdown adapter. Callers receive
citation-substituted Markdown and are responsible for any later CommonMark to
HTML conversion.

#### Citation Syntax

| Syntax | Description |
|---|---|
| `[@key]` | Parenthetical citation |
| `[@key1; @key2]` | Multi-cite cluster |
| `[-@key]` | Suppress-author citation |
| `@key` | Textual / narrative / integral citation |
| `@key [p. 45]` | Textual citation with bracketed locator |
| `[@key, p. 45]` | Parenthetical citation with locator |

Djot and Pandoc-style Markdown both use `@key` as the citation token and `[...]`
for grouping. The Markdown adapter intentionally supports the common subset;
Djot-specific modifiers such as `+` and `!` are not part of Pandoc syntax and
are not supported there.

## Related Crates

- `citum-schema` provides the shared style and reference data model.
- `citum-cli` provides the `citum` command-line interface.
- `citum-server` and `citum-bindings` are adapter surfaces that build on the
  engine API.

## License

Dual-licensed under MIT or Apache-2.0 at your option.
