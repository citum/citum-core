# Per-Document Configuration Overrides Specification

**Status:** Draft
**Date:** 2026-05-26
**Related:** bean `csl26-tap8`, `docs/specs/INTEGRAL_NAME_MEMORY.md`, `docs/specs/UNIFIED_SCOPED_OPTIONS.md`

## Purpose

Authors routinely need to adjust how a citation style renders for a specific
document without forking or modifying a shared style. A thesis submitted to
a German institution may use an English-default style but require German
locale terms. A publisher may deliver manuscripts in a style that prohibits
the 3-em dash convention the house style uses for repeated authors.

This specification defines which style configuration options may be overridden
at the document level, the criteria that determine eligibility, and the three
access surfaces: plain-text document frontmatter, the `citum` CLI, and GUI
consumers such as word-processor plugins.

## Scope

**In scope:**

- Defining the eligible option set and the reasoning behind each inclusion
- YAML frontmatter syntax (`options:` block in document)
- CLI surface (`citum render doc` flags)
- GUI consumer contract (same serde-derived Rust type; YAML/JSON/CBOR share one schema)
- Implementation pattern (sparse-overlay model)
- Relationship between per-document `sort-partitioning` and existing bibliography groups

**Out of scope:**

- Entry-spacing and hanging-indent controls — these are layout concerns owned
  by the rendering target (CSS, LaTeX document class), not the citation
  engine. The engine's output is semantic; physical spacing belongs to the
  consuming layer.
- Disambiguation behavior overrides — disambiguation state is built
  incrementally as the processor walks all citations in the document.
  Allowing a per-document policy change would require re-running
  disambiguation from scratch, and the result would be inconsistent with
  any pre-rendered or cached citation output. This is a processor concern,
  not an author-facing override.
- Any option under `config.processing`, `config.contributors`, `config.dates`,
  `config.titles`, `config.locators`, `config.substitute`, or
  `config.multilingual` — these are structural: they change *what gets cited*
  and *how contributor data is assembled*, not how the output looks. Overriding
  them per-document would silently produce output that misrepresents the
  applied style.

## Eligibility Criteria

An option qualifies for per-document override if and only if:

1. **Presentation-layer**: it controls *how output looks* — punctuation,
   label shape, ordering of pre-assembled elements, locale terms — rather
   than *what gets cited* or *how contributor data is assembled*.
2. **No processor side-effects**: applying it at the end of style resolution,
   after the processor has finished disambiguating and sorting, produces
   correct and consistent output. Options that require the processor to
   re-walk citations do not qualify.
3. **Documented venue variation**: there is evidence from academic publishing
   practice, style manuals, or community tooling that the option genuinely
   varies by document type or venue while the underlying citation logic
   remains the same.

## Eligible Options

### `locale` (top level)

A document may use an English-default style but require output in another
language. Without a per-document locale, the only recourse is to fork the
style and change `info.default-locale` — a maintenance burden that breaks
style updates.

`locale: de-DE` in frontmatter selects the base locale for the document,
meaning all locale-backed terms, date formats, and punctuation rules are
drawn from the German locale rather than the style's default. This is
distinct from the style-internal `config.locale-override`, which is a
patch ID (e.g. `de-DE-chicago`) that loads a partial diff over the resolved
base locale. The document-level `locale` field replaces the base entirely.

Implementation note: the resolver will need a new code path for document-level
base-locale selection, separate from the existing `locale_override` patch
mechanism in `crates/citum-cli/src/style_resolver.rs`.

### `integral-name-memory` (top level)

Already implemented. See `docs/specs/INTEGRAL_NAME_MEMORY.md`.

This option stays at the top level of the `options:` block for backward
compatibility: the current `DocumentFrontmatter.integral_name_memory` field
is a sibling of `bibliography:`, not nested under `options:`. Both locations
must continue to work. When both are present, `options.integral-name-memory`
takes precedence. The `options:` placement is the preferred form going forward.

Document overrides support an additional `enabled: false` switch to suppress
the style's memory policy for that document.

### `bibliography.sort-partitioning`

The style-level `BibliographySortPartitioning` controls automatic grouping of
a multilingual bibliography by Unicode script (`by: script`) or by item
language (`by: language`). A document may need to override this policy —
for example, a multilingual anthology may want script-based sorting disabled
for a chapter that contains only Western-script sources, or may need to adjust
the section headings the style provides.

Per-document `sort-partitioning` overrides the style's multilingual
partitioning policy using the same shape as the style option. All fields are
optional; absent fields inherit the style's defaults.

Valid `by` values are `script` and `language`. The heading shape follows
`BibliographyPartitionHeading`: `literal`, `term`, or `localized`.

#### Relationship to bibliography groups

General-purpose bibliography divisions — primary vs. secondary sources, legal
materials by category (cases / legislation / commentary), print vs. online —
are **not** sort-partitioning. They are handled by the existing
`DocumentFrontmatter.bibliography` field, which accepts a list of
`BibliographyGroup` entries. Each group uses a `GroupSelector` (by type,
cited status, or field value), an optional heading, an optional per-group
sort, and an optional per-group template.

OSCOLA's requirement for separate sections for cases, legislation, and
secondary sources is expressed as bibliography groups with type selectors, not
as sort-partitioning. Chicago divided bibliographies for large theses are
likewise bibliography groups. Sort-partitioning is specifically the mechanism
for script/language-based multilingual reordering.

This distinction resolves the naming confusion between the two features: they
have different shapes, different selectors, and different purposes.

#### Interaction with numeric `label-mode`

When a bibliography uses numeric labels and is also partitioned by script or
language, numeric labels must continue across partitions. Each reference
receives a unique number in document order regardless of which partition it
falls in; partitions affect only the visual grouping and headings. Restarting
labels per partition would make cross-referencing ambiguous (the same number
appearing in multiple sections).

### `bibliography.repeated-author-rendering`

The 3-em dash convention (replacing a repeated author name with ———) varies
substantially by venue. CMOS 14.67 notes that the convention is a *publisher's
prerogative*, and explicitly discourages authors from applying it themselves
in manuscripts submitted for publication — publishers add it in copyediting.
Some publishers (notably Bloomsbury) prohibit it entirely for XML and eBook
workflows, where a dash cannot be reliably expanded back to the author name.

Because the same underlying style (e.g., Chicago author-date) may be used
both by authors preparing manuscripts (where the dash should be suppressed)
and by publishers rendering final output (where the dash is appropriate),
this is a clear case for per-document control.

Values: `full` (always render the full contributor list), `dash`, `dash-with-space`.

### `bibliography.label-mode`

The numeric vs. author-date vs. none label distinction is a document-class
concern. A researcher may use the same style to produce a journal preprint
(no labels, author-date in-text), a conference proceedings draft (numeric
labels required by the venue), and a thesis (author-date labels in the
bibliography matching the in-text key).

Changing label mode does not affect how contributor data is assembled or how
citation disambiguation works — it affects only whether and how a label
component is injected into the bibliography template.

### `bibliography.label-wrap`

Label punctuation — whether a numeric label appears as `[1]`, `(1)`, `1.`,
or as a superscript — varies by venue even within a style family.

**Caveat:** In some style families, the bracket convention is a *defining*
feature. Overriding `label-wrap` per document can produce output that
superficially resembles a different named style without declaring it as such,
which may mislead readers about the citation convention in use. Authors should
use this with care.

## Future Consideration

The following options are plausible candidates for a future revision but are
not included in the initial eligible set. Analysis is preserved here to inform
that decision.

### `bibliography.date-position`

The position of the issued date within a bibliography entry varies by style
family and publisher. However, in author-date styles, date position carries
semantic weight: readers expect to find the date immediately after the author
name so they can match the in-text citation key `(Smith 2020)` to the
bibliography entry. Overriding this in that context degrades usability without
changing correctness. Safe exposure requires detecting whether the resolved
style uses author-date processing — a dependency not yet addressed.

### `bibliography.title-terminator`

Meets the eligibility criteria in isolation but is deferred because its
interaction with `date-position` is non-trivial: a title ending with a period
before a repositioned date can produce double punctuation. Address alongside
`date-position` if both are admitted in a future revision.

### `citation.label-wrap`

Same rationale as `bibliography.label-wrap`. Deferred pending evidence of
demand for independent control over citation-level vs. bibliography-level
wrapping, and observation of `bibliography.label-wrap` usage in practice.

### `citation.group-delimiter`

The delimiter between co-cited references in a single citation cluster varies
by publisher. It does not substitute for style families that differ in other
ways (numbering scheme, date format, title casing, contributor rendering —
the delimiter is one small facet). Deferred pending evidence of author demand
at the document level.

## Access Surfaces

### Plain-text documents (frontmatter)

The primary authoring surface. An `options:` block in document frontmatter
(Djot or Markdown) is parsed automatically by `citum render doc`. All fields
are optional; absent fields inherit the style's defaults.

```yaml
---
options:
  locale: de-DE
  integral-name-memory:
    enabled: false
  bibliography:
    sort-partitioning:
      by: script
      mode: sort-and-sections
      headings:
        latin:
          literal: "Latin-script sources"
        han:
          localized:
            zh: "中文文献"
            en-US: "Chinese-script sources"
    label-mode: numeric
    label-wrap: brackets
    repeated-author-rendering: full
---
```

`integral-name-memory` is also accepted at the document top level (sibling
of `bibliography:`) for backward compatibility:

```yaml
---
integral-name-memory:
  enabled: false
bibliography:
  - id: primary
    ...
---
```

### CLI (`citum render doc`)

`citum render doc` reads frontmatter `options:` automatically. No extra flags
are required for most overrides.

The one exception is locale: authors often need to render a document in a
different language without modifying the document itself (e.g., batch-rendering
the same source for different regional audiences, or when the document comes
from an external source). A `--locale / -L` flag will be added to
`RenderDocArgs`, matching the existing `-L/--locale` flag on `render refs`:

```
citum render doc manuscript.djot -s apa-7th -b refs.json --locale de-DE
```

CLI `--locale` takes precedence over a `locale:` field in frontmatter, which
takes precedence over the style's default locale.

No other per-document options get dedicated CLI flags in v1. Options that vary
per document (label mode, repeated-author rendering, etc.) belong in frontmatter
where they are version-controlled alongside the document.

### GUI consumers (word-processor plugins)

The `DocumentOptionsOverride` Rust struct is the single contract for all
consumers. Serde derives YAML, JSON, and CBOR representations from one type,
so plain-text frontmatter, JSON-posting GUI plugins, and binary CBOR protocols
all share the same schema without a separate wire-format contract.

## Implementation Pattern

The sparse-overlay model used for `integral-name-memory` extends to the full
`options:` block:

1. **`DocumentOptionsOverride` struct** (new) in
   `crates/citum-engine/src/processor/document/types.rs` — mirrors the fields
   in the eligible set, all `Option<T>`. Implements an `apply_to(style)` method
   that writes non-`None` fields into the resolved style's configuration after
   normal scoped-options application.

2. **`DocumentFrontmatter` extension** in
   `crates/citum-engine/src/processor/document/djot/parsing.rs` — adds
   `pub options: Option<DocumentOptionsOverride>`.

3. **`process_document()` merge step** in
   `crates/citum-engine/src/processor/document/pipeline.rs` — calls
   `options.apply_to(&mut style)` after the style is resolved and before the
   processor runs. This keeps the document override in the same phase as
   the existing `processor_with_document_integral_name_override()` call.

4. **`--locale` flag** added to `RenderDocArgs` in
   `crates/citum-cli/src/args.rs`, wired through `render/mod.rs` into the
   processor setup, mirroring the existing `RenderRefsArgs.locale` path.

5. **Document locale resolver** (new) in `crates/citum-cli/src/style_resolver.rs`
   — a separate code path from `locale_override` that selects the full base
   locale by BCP 47 ID rather than loading a patch file.

The legacy top-level `integral-name-memory:` field in `DocumentFrontmatter`
is preserved; `DocumentOptionsOverride.integral_name_memory` takes precedence
when both are set.

## Acceptance Criteria

- [ ] `DocumentFrontmatter` deserializes an `options:` block with any subset
      of the eligible fields; unknown keys are rejected.
- [ ] Each eligible field, when set, overrides the corresponding style option
      and produces the correct rendering output.
- [ ] Absent fields do not change the style's defaults.
- [ ] Legacy top-level `integral-name-memory:` continues to work alongside
      `options.integral-name-memory:`; the latter takes precedence when both
      are present.
- [ ] `bibliography.sort-partitioning` accepts `by: script | language` and
      `BibliographyPartitionHeading` shapes (`literal`, `term`, `localized`).
- [ ] Setting `sort-partitioning` with `label-mode: numeric` produces
      continuous label numbering across partitions.
- [ ] `citum render doc --locale de-DE` selects the German base locale and
      takes precedence over any `locale:` in frontmatter.
- [ ] GUI consumers sending the equivalent JSON shape receive identical output.

## Changelog

- 2026-05-26: Address Copilot review — clarify sort-partitioning as multilingual-only
  (distinct from bibliography groups); rename locale-override → locale (base-locale
  selector, not patch ID); add CLI surface section; fix integral-name-memory compat note.
- 2026-05-26: Address PR review — narrow eligible set, rename sort-partitioning
  → sections (reverted), establish serde-based single contract, move
  date-position / title-terminator / citation overrides to Future Consideration.
- 2026-05-26: Initial draft.
