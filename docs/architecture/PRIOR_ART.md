# Prior Art Reference

This document summarizes key patterns from existing bibliography systems that Citum should follow or learn from. Refer to this when designing new features.

---

## Sources

| System | Type | Key Strengths | Documentation |
|--------|------|---------------|---------------|
| **CSL 1.0** | XML schema | Established vocabulary, 2,844+ styles, locale system | [CSL Spec](https://docs.citationstyles.org/en/stable/specification.html) |
| **CSL-M** | CSL extension | Legal citations, multilingual, institutional names | [citeproc-js docs](https://citeproc-js.readthedocs.io/en/latest/csl-m/) |
| **biblatex** | LaTeX package | Flat options, EDTF dates, sorting, biber backend | [CTAN biblatex](https://ctan.org/pkg/biblatex) |
| **citeproc-rs** | Rust impl | Interactive architecture, salsa incremental, WASM | [GitHub](https://github.com/zotero/citeproc-rs) |

### citeproc-rs (Zotero)

A Rust CSL implementation funded by Zotero, now unmaintained. Key architectural ideas:

1. **Salsa incremental computation**: Uses `salsa` crate for demand-driven, incremental processing—ideal for interactive apps where edits should only recompute affected citations
2. **WASM-first design**: Built with WebAssembly bindings for browser/Zotero integration
3. **Modular crate structure**: `csl` (parsing), `db` (state), `proc` (processing), `io` (formats)
4. **Disambiguation graph**: Visual graph-based approach to cite ambiguity resolution

**Ideas to study — do not copy code.** citeproc-rs is MPL-2.0. Citum is `MIT OR Apache-2.0`. These licenses are incompatible for direct code vendoring (MPL-2.0 is file-level copyleft). The following areas are valuable for clean-room design inspiration:

- Name parsing/formatting logic
- Disambiguation algorithms
- Locale merging/fallback
- Number/page-range formatting

**Lessons from its "failure"**:
- Full CSL 1.0 compatibility is extremely complex
- CSL-M support adds significant scope
- Interactive vs batch processing have different architectural needs

---

## Citum-Specific Design Goals

These goals were identified early in the project. Each item now links to the spec or implementation where it has been realized.

### Presets
Pervasive presets that bundle common configurations, avoiding macro complexity. Realized — see [`docs/specs/STYLE_PRESET_ARCHITECTURE.md`](../specs/STYLE_PRESET_ARCHITECTURE.md) (Active).

```yaml
title: ABC Journal
processing: author-date  # preset
dates: long              # preset
contributors: long       # preset
titles: apa              # preset
citation:
  template:
    - contributor: author
```

**Rationale**: Most styles are variations on a few base patterns. Presets reduce authoring friction.

### Hyperlinks
Declarative link configuration — still in design; no spec yet.

```yaml
links:
  target: url-or-doi      # Use URL if present, else construct from DOI
  anchor: title           # Which element becomes the clickable link
```

**Prior art**: CSL Appendix VI discusses links but lacks declarative control.

### Djot Integration
Use Djot (a markdown dialect) for:
1. Document markup with citations
2. Rich text within fields (titles with math, emphasis)

Realized — see [`docs/specs/DJOT_RICH_TEXT.md`](../specs/DJOT_RICH_TEXT.md) (Active).

**Rationale**: Cleaner than CSL 1.0's embedded HTML; aligns with modern markup.

### Pluggable Renderers
Trait-based output renderers (HTML, RTF, LaTeX, Typst, Djot). Implemented — see `crates/citum-engine/src/render/` (HTML, LaTeX, Markdown, Djot, and format-neutral markup modules).

```rust
pub trait Renderer {
    type Output: Display;
    fn render(&self, table: &Table) -> Self::Output;
}
```

**Prior art**:
- jotdown's `Render` trait
- stanza's markdown renderer
- citeproc-rs `OutputFormat` trait

---

## Key Patterns to Borrow

### 1. Flat Options Architecture (biblatex)

biblatex uses completely flat parameters scoped to different levels. Citum mirrors this.

```
Global → Per-Type → Per-Entry → Per-Field
```

**biblatex example:**
```latex
\usepackage[
  maxnames=3,          % global
  maxbibnames=99,      % bibliography scope
  maxcitenames=2,      % citation scope
]{biblatex}
```

**Citum equivalent:**
```yaml
options:
  contributors:
    shorten:
      min: 4
      use-first: 1
citation:
  options:
    contributors:
      shorten:
        use-first: 2
```

**Lesson**: Separate citation-context options from bibliography-context options.

---

### 2. Locale-Specific Layouts (CSL-M)

CSL-M allows multiple `<cs:layout>` elements with locale targeting:

```xml
<citation>
  <layout locale="en es de">
    <text macro="layout-citation-roman"/>
  </layout>
  <layout locale="ja zh">
    <text macro="layout-citation-cjk"/>
  </layout>
  <layout>
    <!-- Default fallback -->
  </layout>
</citation>
```

**Use case**: Japanese academic bibliography citing both English and Japanese sources with appropriate conventions for each.

**Citum approach**: Realized — see [`docs/specs/MULTILINGUAL.md`](../specs/MULTILINGUAL.md) (Active).

```yaml
bibliography:
  locales:
    - locale: [ja, zh]
      template:
        - contributor: author
          delimiter: "・"
    - default:
      template:
        - contributor: author
          delimiter: ", "
```

---

### 3. Entry-Level Language (biblatex + CSL-M)

Both systems support language tagging at the entry level:

**biblatex:**
```bibtex
@book{example,
  title = {المرجع فى قواعد اللغة القبطية},
  langid = {arabic}
}
```

**CSL-M:**
- `language` variable on items
- Matched against `locale` attribute on layouts
- Affects which locale terms are used

**Citum approach**: Realized. Add `language` to CSL-JSON input; used for:
1. Selecting locale-specific templates
2. Applying locale terms
3. Sorting collation

See [`docs/specs/MULTILINGUAL.md`](../specs/MULTILINGUAL.md) and [`docs/specs/UNICODE_BIBLIOGRAPHY_SORTING.md`](../specs/UNICODE_BIBLIOGRAPHY_SORTING.md).

---

### 4. Legal Citation Extensions (CSL-M)

CSL-M adds essential legal features. Citum support is in design — see [`docs/specs/LEGAL_CITATIONS.md`](../specs/LEGAL_CITATIONS.md) (Design Phase).

#### Extended Types
| Type | Use Case |
|------|----------|
| `hearing` | Government committee transcripts |
| `regulation` | Administrative orders |
| `classic` | Commonly cited sources (Bible, Aristotle) |

#### Jurisdiction Hierarchy
```
us → us:federal → us:federal:circuit:9
```

Used with `court-class` for grouping courts by hierarchy.

#### Parallel Citations
```xml
<text variable="title" parallel-first="title"/>
<text variable="container-title" parallel-last="container-title"/>
```

First cite shows full form; subsequent show short form.

#### Position Extensions
| Position | Meaning |
|----------|---------|
| `far-note` | Cited before, but not recently |
| `container-subsequent` | Same container cited before |

---

### 5. EDTF Dates (biblatex/biber)

biblatex with biber backend natively supports EDTF:

```bibtex
@book{example,
  date = {1990~/2000?}  % Approximate start, uncertain end
}
```

Features:
- Uncertainty markers: `?` (uncertain), `~` (approximate)
- Ranges: `1990/2000`
- Open ranges: `1990/..` (ongoing)
- Precision levels: year, month, day

**Citum**: Realized — see [`docs/specs/EDTF_HISTORICAL_ERA_RENDERING.md`](../specs/EDTF_HISTORICAL_ERA_RENDERING.md) (Active) for the historical-era rendering contract and tracked follow-up work.

---

### 6. Sorting Templates (biblatex)

biblatex provides shorthand sorting schemes:

| Shorthand | Meaning |
|-----------|---------|
| `nty` | name, title, year |
| `nyt` | name, year, title |
| `ynt` | year, name, title |
| `none` | citation order |

Plus explicit override fields:
- `sortname` - for sorting only (not display)
- `sorttitle` - ditto
- `sortyear` - ditto

**Citum approach**: Realized — see [`docs/specs/SORTING.md`](../specs/SORTING.md) (Active).

```yaml
sort:
  shorten-names: true
  template:
    - key: author
    - key: year
      ascending: false
```

---

### 7. Institutional Names (CSL-M)

CSL-M provides `<cs:institution>` for complex organizational names:

```xml
<names variable="author">
  <name/>
  <institution
    delimiter=", "
    use-first="1"
    use-last="1"
    institution-parts="short-long">
    <institution-part name="long" if-short="true" prefix=" (" suffix=")"/>
  </institution>
</names>
```

Renders: "WHO (World Health Organization)"

**Features:**
- `use-first`, `use-last` - truncate hierarchy
- `institution-parts` - long, short, short-long, long-short
- `if-short` - conditional on abbreviation availability

See also [`docs/specs/MULTILINGUAL_NAMES.md`](../specs/MULTILINGUAL_NAMES.md).

---

## Anti-Patterns to Avoid

### 1. Procedural Conditionals (CSL 1.0)

**Don't do this:**
```xml
<choose>
  <if type="article-journal">
    <text variable="container-title" font-style="italic"/>
  </if>
  <else-if type="chapter">
    <text variable="container-title" font-style="italic" prefix="In "/>
  </else-if>
  <!-- 20 more branches... -->
</choose>
```

**Do this instead (Citum):**
```yaml
- title: parent-serial
  emph: true
  overrides:
    chapter:
      prefix: "In "
```

### 2. Implicit Magic in Processor

**Don't do this:**
```rust
// Hidden in processor code
if ref_type == "article-journal" {
    separator = ", ";
}
```

**Do this instead:**
Express all behavior in the style YAML. The processor should be "dumb."

### 3. Over-Engineering Multilingual

CSL-M's deprecated `alt-*` extensions (`alt-title`, `alt-container-title`) were too complex.

**Better approach:**
- Entry-level `language` field
- Locale-specific template sections
- Field-level language scoping only where essential

---

## Architectural Considerations

### Batch vs Interactive Processing

Citum supports both modes. See [`docs/architecture/CITUM_SERVER_MODE.md`](CITUM_SERVER_MODE.md) and [`docs/specs/SERVER_INTERACTIVE_API.md`](../specs/SERVER_INTERACTIVE_API.md) (Draft) for the interactive path design.

| Mode | Use Case | Characteristics |
|------|----------|-----------------|
| **Batch** | Pandoc, LaTeX, CLI | Process all citations at once, optimize for throughput |
| **Interactive** | Zotero, Word, web apps | Incremental updates, optimize for latency |

**citeproc-rs approach**: Used `salsa` crate for incremental computation:
- Demand-driven: only compute what's needed
- Memoization: cache intermediate results
- Invalidation: track dependencies, recompute on change

**Trade-offs**:
- Salsa adds complexity and compile time
- Batch processing may not benefit from incrementality
- JSON server mode (like Haskell citeproc) is the chosen middle ground — specced as the interactive API

**Current status**: Batch-optimized architecture is shipped. Interactive/server mode is specced and in design; see the specs above for trade-off analysis.

---

## References

- [CSL 1.0.2 Specification](https://docs.citationstyles.org/en/stable/specification.html)
- [CSL-M Extensions](https://citeproc-js.readthedocs.io/en/latest/csl-m/)
- [biblatex Manual](https://ctan.org/pkg/biblatex)
- [citeproc-rs](https://github.com/zotero/citeproc-rs) — Rust CSL impl (unmaintained, MPL-2.0; study only — license-incompatible with Citum)
- Citum specs: [STYLE_PRESET_ARCHITECTURE](../specs/STYLE_PRESET_ARCHITECTURE.md) · [DJOT_RICH_TEXT](../specs/DJOT_RICH_TEXT.md) · [SORTING](../specs/SORTING.md) · [MULTILINGUAL](../specs/MULTILINGUAL.md) · [LEGAL_CITATIONS](../specs/LEGAL_CITATIONS.md) · [EDTF_HISTORICAL_ERA_RENDERING](../specs/EDTF_HISTORICAL_ERA_RENDERING.md) · [DISAMBIGUATION](../specs/DISAMBIGUATION.md) · [SERVER_INTERACTIVE_API](../specs/SERVER_INTERACTIVE_API.md)
