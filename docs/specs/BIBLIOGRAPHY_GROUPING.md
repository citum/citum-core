# Bibliography Grouping Architecture

**Status:** Active
**Created:** 2026-02-15
**Related:** bean `csl26-effd`, `csl26-o8ji`, `csl26-group`;
`docs/specs/PER_DOCUMENT_CONFIG_OVERRIDES.md`,
`docs/specs/SERVER_INTERACTIVE_API.md`

## Overview

This document defines the architecture for configurable bibliography grouping
in Citum. The design enables styles and documents to divide a bibliography
into labelled sections, each with its own selector, sort order, heading, and
optional template override. Use cases include legal citations (type-based
hierarchies), multilingual bibliographies, and primary/secondary source
divisions in historical scholarship.

## Vocabulary

| Term | Meaning |
|---|---|
| **group** | A declarative definition: what belongs in a section, how it is headed, sorted, and rendered (`BibliographyGroup`). |
| **block** | A group placed in document order: the group definition plus a caller-supplied stable identifier (`BibliographyBlockRequest`). |

The same `BibliographyGroup` type is used in the style schema, in document
frontmatter, and (wrapped in a `BibliographyBlockRequest`) in the CLI and
server API. A reader should see "group" where the focus is the definition
and "block" where the focus is placement in a rendered document.

## Design Principles

1. **First-match semantics** — a reference appears in the first group whose
   selector matches it and in no subsequent group. This prevents duplication
   and is essential for legal citation hierarchies.
2. **Explicit over magic** — all grouping is declared in YAML or JSON; no
   hardcoded hierarchies.
3. **Graceful degradation** — omitting groups produces a flat bibliography.
4. **Per-group sorting** — each group may override the global sort with its
   own sort spec, enabling culturally appropriate collation within sections.

## Core Type: `BibliographyGroup`

Canonical Rust definition: `citum_schema::grouping::BibliographyGroup`
(`crates/citum-schema-style/src/grouping.rs`).

```yaml
# Minimal
- id: cases
  selector:
    type: legal-case

# Full
- id: vietnamese
  heading:
    literal: "Tài liệu tiếng Việt"
  selector:
    field:
      language: vi
  sort:
    template:
      - key: author
        sort-order: given-family
  template: ...         # optional entry-template override
  disambiguate: locally # or globally (default)
```

### `GroupSelector`

| Field | Description |
|---|---|
| `type` | Match by reference type (single string or list). |
| `cited` | `visible` (cited in document) or `any` (all references). To select only uncited references, use `not: { cited: visible }`. |
| `field` | Match by arbitrary field value (e.g. `language: vi`). |
| `not` | Negate a nested selector — use for catch-all groups. |

An empty selector (`selector: {}`) matches every reference not yet assigned
to a prior group, making it a natural catch-all.

### `GroupHeading`

One of:
- `literal: "Cases"` — verbatim text.
- `term: bibliography` (with optional sibling `form: long`) — locale-resolved term.
- `localized: { zh: "中文文献", en-US: "Chinese-script sources" }` — per-locale map.

Omit `heading:` entirely to render the group's entries with no section heading.

## Rendering Primitive

**`Processor::render_document_bibliography_blocks`**
(`crates/citum-engine/src/processor/bibliography/grouping.rs`)

All document-level sectional bibliography rendering flows through this one
function. It accepts an ordered `&[BibliographyGroup]` and a shared
`assigned: HashSet<String>` dedup set, and returns a
`Vec<RenderedBibliographyGroup>` — **one element per input group** (including
empty ones). Each element carries a resolved `heading: Option<String>`, a
formatted `body: String`, and an `entries` vec. Callers are responsible for
filtering: the document pipeline skips groups where `entries.is_empty()`; the
server API echoes every requested block by `id` regardless of emptiness.

The dedup set ensures first-match semantics across the full ordered sequence
regardless of which input surface supplied the groups.

## Input Surfaces

All four surfaces resolve to `&[BibliographyGroup]` before reaching the
rendering primitive. Precedence when surfaces conflict is listed highest first.

| Surface | How to use | Wrapper type |
|---|---|---|
| **Caller / CLI / server** | `--bibliography-blocks <json>` on `citum render doc`; `bibliography_blocks` in `FormatDocumentRequest` | `BibliographyBlockRequest { id, group }` array |
| **Fenced-div** | `:::bibliography{#id}` markers embedded in the document body | Parsed by the Djot/Markdown adapter |
| **Frontmatter `bibliography:`** | Top-level YAML list of `BibliographyGroup` objects in document frontmatter | Bare `BibliographyGroup` array |
| **Style `bibliography.groups`** | `groups:` list in the style's `bibliography:` section | Bare `BibliographyGroup` array |

When the frontmatter `bibliography:` list is present, the style-level
`bibliography.groups` is ignored for that document. Caller/CLI/server blocks
and fenced-div blocks are document-level mechanisms that also take precedence
over style-level groups.

### Trailing vs. Positioned rendering

- **Trailing** (frontmatter list and caller/CLI/server blocks): groups are
  rendered as an ordered sequence appended after the document body, under an
  automatic "Bibliography" H1 heading added by the engine. Group headings are
  H2 sub-headings (`## heading` / `\subsection*` / `== heading` / `<h2>`).
- **Positioned** (fenced-div blocks): each `:::bibliography{#id}` marker in
  the body is replaced in-place by its rendered group. No automatic "Bibliography"
  heading is added; authors control placement and headings in the document.
- **API / server** (caller-supplied blocks): sections are appended as trailing
  H2-level sections, with no automatic "Bibliography" wrapper heading. The
  consuming application (e.g. a word processor) controls the top-level heading.

## Grouping vs. Sort-Partitioning

These are two distinct mechanisms and must not be confused.

| Feature | Purpose | Configured via |
|---|---|---|
| **Bibliography groups** | Divide a bibliography into explicitly declared sections by type, language, cited status, or custom field | `bibliography.groups` (style) or `bibliography:` (frontmatter/CLI/server) |
| **Sort-partitioning** | Reorder and visually separate a multilingual bibliography by Unicode script (`script`) or item language (`language`) | `bibliography.options.sort-partitioning` (style) or `options.bibliography.sort-partitioning` (frontmatter) |

Sort-partitioning is automatic and data-driven (the engine derives partition
keys from reference metadata). Groups are explicit (the author or style designer
declares each section). A bibliography can use both: groups divide by type,
and sort-partitioning further reorders within a group by script.

When `label-mode: numeric` is active alongside partitioning, numeric labels
run continuously across all partitions; restarting per partition would make
cross-references ambiguous.

## Style-Level Groups

```yaml
bibliography:
  groups:
    - id: cases
      heading:
        literal: "Cases"
      selector:
        type: legal-case
      sort:
        template:
          - key: field
            field: court-class
            order: [supreme, appellate, trial]
          - key: issued
            ascending: false

    - id: statutes
      heading:
        literal: "Statutes"
      selector:
        type: statute
      sort:
        template:
          - key: title

    - id: other
      heading:
        literal: "Secondary Sources"
      selector:
        not:
          type: [legal-case, statute]
```

Style-level groups apply to every document rendered with that style unless
the document provides its own frontmatter `bibliography:` list or caller blocks.

## Per-Document Frontmatter Groups

```yaml
---
bibliography:
  - id: primary
    heading:
      literal: "Primary Sources"
    selector:
      type: [manuscript, interview, archival-document]

  - id: secondary
    heading:
      literal: "Secondary Sources"
    selector:
      not:
        type: [manuscript, interview, archival-document]
---
```

**Unmatched entries are omitted.** The `not:` catch-all pattern above ensures
all references appear. Without it, references not matching any group selector
are silently skipped. This is intentional and consistent with how caller-supplied
blocks work — use an explicit catch-all group to surface unmatched references.

See `docs/specs/PER_DOCUMENT_CONFIG_OVERRIDES.md` for the full set of per-document
override surfaces, including CLI flags and the server API.

## Multilingual Sorting Example (Juris-M)

```yaml
bibliography:
  groups:
    - id: vietnamese
      heading:
        literal: "Tài liệu tiếng Việt"
      selector:
        field:
          language: vi
      sort:
        template:
          - key: author
            sort-order: given-family

    - id: western
      selector:
        not:
          field:
            language: vi
      sort:
        template:
          - key: author
            sort-order: family-given
```

## Open Questions

**Q: Should items appear in multiple groups?**
**A:** No. First-match semantics prevent duplication. This is critical for
legal citations where the same item must not appear in both "Cases" and
"Secondary Sources."

**Q: Should nested groups be supported?**
**A:** No. Flat groups with selectors are sufficient for all known use cases.
Nesting adds complexity without clear benefit.

**Q: What about items that match no group?**
**A:** They are omitted. Use an explicit catch-all group with `selector: {}`
or a `not:` negation to capture remaining entries.

## Prior Art

- **biblatex** `\printbibliography[type=article,title=Articles]` — flat,
  declarative, negation for catch-all. The Citum group selector generalises
  this to multi-field predicates.
- **CSL 1.0** — no grouping constructs; all grouping is hardcoded in processors.
- **Juris-M / CSL-M** — locale-specific layout elements, but no general grouping.

## Compliance with Citum Principles

- **Explicit Over Magic:** all grouping declared in YAML or JSON.
- **Declarative Templates:** flat selector syntax, no procedural conditionals.
- **Code-as-Schema:** `BibliographyGroup` Rust struct drives JSON Schema.
- **Graceful Degradation:** omitting `groups` produces a flat bibliography.
- **Multilingual Support:** per-group sorting enables culturally appropriate collation.

## Changelog

- 2026-06-08: Status Design → Active. Rewrite to reflect implemented code:
  canonical type, `render_document_bibliography_blocks` primitive, vocabulary
  rule (group vs block), input-surfaces table, grouping-vs-partitioning
  distinction, unmatched-entry policy. Remove speculative Rust/processor
  sections. (csl26-effd, csl26-o8ji, fd0c6eee)
- 2026-02-15: Initial draft.
