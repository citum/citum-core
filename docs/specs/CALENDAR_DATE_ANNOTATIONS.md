# Calendar Date Annotations Specification

**Status:** Active
**Version:** 1.6
**Date:** 2026-07-20
**Related:** [Date Model](./DATE_MODEL.md), [GB/T 7714—2025 Citation Conventions](../reference/GBT_7714_CITATION_CONVENTIONS.md), bean `csl26-0kqf` (the separate, still-unimplemented computed regnal-year feature this data model unblocks), bean `csl26-k2kp` (script-aware wrap renderer, completed; spec `docs/specs/PUNCTUATION_REALIZATION.md`), [PR #1067 discussion](https://github.com/citum/citum-core/pull/1067#issuecomment-5011594655)

## Purpose

Define an opaque `note` sub-field on any Citum date value, and a
style-controlled wrap for rendering it, so a reference can retain
source-calendar wording alongside its canonical EDTF value. The motivating
GB/T 7714—2025 examples keep a sortable Gregorian year while showing the
source calendar wording, such as `1947（民国三十六年）` and
`1705（康熙四十四年）`. Calendar annotation is the first consumer of this
capability, not a special case of it: `note` is a general opaque-text
sub-field applicable to any date, not a calendar-specific type.

## Scope

In scope: a backward-compatible annotated date value for every Citum date
field, value-only processing semantics, an explicit style opt-in for
rendering, legacy CSL/Zotero-extra conversion for the GB/T examples, and
focused standard-derived verification.

Out of scope: calendar-system identifiers, calendar conversion, dynasty or
era lookup tables, validation or generation of the note text, changes to
EDTF parsing, automatic display in existing styles, leading (before-date)
annotation placement, inverse-primary presentations where the Gregorian value
rather than the note is wrapped (e.g. `民国三十六年（1947）`), and the
broader GB/T author-date fidelity tune tracked by `csl26-6eak`.

## Design

### Input model

Every Citum date field accepts either the existing EDTF scalar or a structured
date value:

```yaml
issued:
  value: "1947"
  note: "民国三十六年"
```

The equivalent unannotated input remains:

```yaml
issued: "1947"
```

The structured form has exactly two fields:

- `value` is required and contains the canonical EDTF value.
- `note` is optional opaque Unicode text supplied by the data producer.

Unknown fields are rejected. A structured value without `note` is valid but
canonical serialization uses the scalar form. Annotated values serialize as
mappings. This preserves existing input and output shapes when no annotation
is present.

The model applies uniformly to `issued`, `created`, `accessed`, and every
other Citum date field. GB/T publication dates are the first supported use,
not a special type in the data model.

`note` contains only the text inside its eventual wrapper. Producers write
`民国三十六年`, not `（民国三十六年）`. Citum preserves the text verbatim and
does not identify, parse, translate, normalize, or verify its calendar
system. In particular, the model has no `system`, `original-calendar`,
dynasty, emperor, or era-name field.

**Not the CSL `note` variable.** CSL and Zotero define a top-level `note`
variable on the reference (free-text annotation about the work). Citum's
`note` here is a sub-field of a *date value* (`issued.note`, `created.note`,
…), a different and narrower scope. The two do not collide in the schema,
but authors and reviewers should not conflate them.

### Processing semantics

`value` is the only computational date. The following operations ignore
`note`:

- bibliography and citation sorting
- author-date collision grouping and disambiguation
- year-suffix assignment order
- date fallback selection
- EDTF uncertainty, approximation, interval, and historical-era handling

Two references with the same author and `value` but different notes remain
an author-date collision. A note cannot make otherwise equal dates distinct.
Conversely, the lexical content of a note cannot reorder references.

### Public data type

The canonical date type used by every date field on every reference struct
(`crates/citum-schema-data/src/reference/date.rs`) is renamed `EdtfString` →
`DateValue` and gains the note field in place, rather than introducing a
parallel wrapper type around it:

```rust
pub struct DateValue {   // was: struct EdtfString(pub String)
    pub value: String,   // the EDTF value (was the tuple's .0)
    pub note: Option<String>,
}
```

This changes zero field declarations — `issued: EdtfString`, `accessed:
Option<EdtfString>`, etc. on every reference struct become `issued:
DateValue`, `accessed: Option<DateValue>` via a token rename, not a
per-field type change. Custom scalar/mapping `Serialize`/`Deserialize` and
`JsonSchema` are implemented once on `DateValue` itself. All public items
receive Rust documentation. Existing value-oriented accessors (`parse()`,
`year()`, `is_range()`, …) continue to expose the EDTF value used by
processors, reading `self.value` in place of the old `self.0`. Rendering
gains access to the complete `DateValue`, including `note`.

### Style opt-in

Notes are hidden unless a style enables them through a single `note-wrap`
option in its **date** configuration. The option reuses the existing `wrap`
value model — a `WrapConfig`, given either as a shorthand (`parentheses`,
`brackets`, `quotes`) or the expanded mapping form — rather than inventing a
new wrapper vocabulary:

```yaml
bibliography:
  options:
    dates:
      note-wrap: parentheses
```

`DateConfig.note_wrap` (`crates/citum-schema-style/src/options/dates.rs`) is
an optional `WrapConfig`, a flat field alongside `DateConfig`'s other
date-rendering knobs (`era-labels`, `range-delimiter`,
`approximation-marker`). When absent, notes are hidden. When present, every
date the style renders in that scope wraps its `note` (when the input has
one) in the configured punctuation, appended after the complete formatted
date. This is a single style-level setting, not a per-`TemplateDate` field,
so it never has to be repeated across date components.

**Extension point: a bare (unwrapped) note.** `WrapPunctuation` currently
has three variants — `parentheses`, `brackets`, `quotes` — so `note-wrap`
can hide the note (option absent) or wrap it in one of those three, but
cannot render it delimiter-free, e.g. `1947 民国三十六年` rather than
`1947（民国三十六年）`. Not attempted here — no known GB/T or other style
needs it — but a later revision could add a `WrapPunctuation::None`
(or similarly named) variant, `realize_wrap`-handled as an empty
open/close pair. That would extend the *general* wrap vocabulary (shared
by every `wrap:`-using template construct, not a note-specific field), so
existing behavior is unaffected; the caller would still need to opt into
a separator explicitly via `note-wrap: { punctuation: none, inner-prefix:
" " }` — inner-prefix/suffix already exist for exactly this purpose and
are not auto-inferred.

`note-wrap` lives on `DateConfig`, not `MultilingualConfig`: it is a
date-rendering concern, and script-appropriate delimiter width is handled
independently by the realization layer described below — the option itself
carries no multilingual-specific state.

**Bibliography-only scoping.** `DateConfig` is embedded at three scopes — the
global `Config`, `CitationOptions`, and `BibliographyOptions`
(`crates/citum-schema-style/src/options/mod.rs`), matching the pattern
already used for `dates.era-labels` and friends. Setting `note-wrap` under
the **bibliography** options block (as above) scopes note rendering to the
bibliography alone; citations carry no `note-wrap` and therefore render no
note. GB/T author-date citations keep their citeproc-js parity with no
per-component opt-in.

**Script-appropriate delimiters.** The width of the delimiters is not
authored; it follows the item's script. The `csl26-k2kp` increment-1
script-aware wrap renderer (completed; spec `PUNCTUATION_REALIZATION.md`)
already realizes `wrap: parentheses` as full-width `（ ）` for a CJK-script
item and half-width `( )` for a Latin-script item in the same bilingual
bibliography, via `realize_wrap` (`crates/citum-engine/src/render/format.rs`).
This spec's rendering reuses that path directly; it adds no separate
half-width → full-width normalization.

The renderer:

1. formats `value` normally, including EDTF markers and any year suffix;
2. wraps the `note` text with the `note-wrap` `WrapConfig`, rendered
   script-appropriately via `realize_wrap` and escaped through the active
   output format; and
3. appends the wrapped note directly to the complete formatted date before
   the date component's outer prefix, suffix, or wrap is applied.

The note follows the complete date or interval, not an internal year token.
Trailing placement is deliberate: GB/T 7714 places the annotation last and is
the only attested convention. A leading annotation is out of scope, but could
be added later without a data-model change via an optional
`position: before | after` field on `note-wrap` (default `after`).
With `note-wrap: parentheses` in the bibliography scope, CJK-script
records render:

| Value | Note | Result |
|---|---|---|
| `1947` | `民国三十六年` | `1947（民国三十六年）` |
| `1947` with suffix `a` | `民国三十六年` | `1947a（民国三十六年）` |
| `1947-05-01` | `民国三十六年` | `1947-05-01（民国三十六年）` |
| `1705/1706` | `康熙四十四年至四十五年` | `1705—1706（康熙四十四年至四十五年）` |

The GB/T bibliography opts in; its author-date citations and unrelated styles
do not.

### Legacy input conversion

The CSL/Zotero-extra convention used by the pinned GB/T fixture stores the
annotation in a note while retaining a structured Gregorian year:

```text
issued: 1947（民国三十六年）
```

Conversion recognizes the bounded shape
`<EDTF value>（<non-empty note>）` using **full-width** parentheses only (not
half-width `( )`), so it stays disjoint from the `c1988`, `1995印刷`, and
`1936~` publication-year substitutes and never misfires on Latin-script data.
It keeps the EDTF portion as `value` and stores the parenthesis contents as
`note`. It does not perform a calendar conversion or general-purpose parsing
of arbitrary date prose.
Existing handling for copyright years, printing years, and approximate EDTF
dates remains authoritative and unchanged.

This is CSL-JSON/Zotero-extra **reference-data** conversion (an unparsed
`issued:` note-field override coexisting with an already-structured `issued`
date, surfaced generically as the `issued-note-literal` extra by
`csl-legacy`), not style migration — it lives alongside
`copyright_year_from_legacy`/`printing_year_from_legacy` in
`crates/citum-schema-data/src/reference/conversion/mod.rs`
(`annotated_issued_from_legacy`), not in `citum-migrate` (which converts CSL
XML styles, a different concern).

### GB/T author-date and verification overlap

The pinned GB/T author-date corpus already contains the two §7.5.4.1
records. A baseline audit found:

- all eight author-date citation scenarios pass;
- the still-untuned bibliography passes 0 of 203 citeproc-js comparisons;
- both citeproc-js and current Citum output only the Gregorian years `1705`
  and `1947` for the two target records.

The source CSL-M style therefore cannot be the authority for this feature:
matching it would preserve the missing annotation. Implementation adds
focused expectations derived from GB/T 7714—2025 and registers the two
intentional citeproc-js divergences. The broader standard-authority benchmark
remains tracked by `csl26-9qat`; general author-date bibliography tuning
remains tracked by `csl26-6eak`.

## Implementation Notes

- Implement scalar/mapping serde on `DateValue`; do not make every containing
  reference type hand-write the same compatibility logic.
- Route existing EDTF parsing and value-oriented accessors through
  `DateValue.value` so sorting and disambiguation do not acquire annotation
  dependencies.
- Add `note_wrap: Option<WrapConfig>` to `DateConfig`
  (`crates/citum-schema-style/src/options/dates.rs`) and read it from the
  bibliography scope so citations are unaffected without a separate toggle.
- Apply the `note-wrap` `WrapConfig` through the output-format abstraction
  (`realize_wrap`, `crates/citum-engine/src/render/format.rs`, used by every
  `render/*.rs` output format) so HTML, Djot, Markdown, LaTeX, plain text,
  Typst, and other renderers escape the note text consistently and pick up
  script-aware full-width CJK delimiters for free. Do not add a local
  half-width → full-width normalization here.
- Enable the option in the GB/T style's bibliography `dates` options only.
- Regenerate checked-in JSON schemas with `just schema-gen` when the data and
  style types are implemented.

## Acceptance Criteria

- [x] Scalar EDTF dates retain their existing YAML/JSON wire shape and
  round-trip behavior.
- [x] Existing serialized reference corpora round-trip byte-identically after
  the `EdtfString` → `DateValue` rename, with the legacy-conversion path
  (`crates/citum-schema-data/src/reference/conversion/mod.rs`) exercised for
  the change.
- [x] Every Citum date field accepts the structured `{ value, note }` form
  and rejects unknown mapping fields.
- [x] `note` is preserved verbatim and does not affect sorting, fallback
  selection, collision grouping, or year-suffix assignment.
- [x] A style without `note-wrap` renders only the EDTF value.
- [x] A style with `note-wrap: parentheses` in its bibliography `dates`
  options renders the Minguo and Kangxi examples with the wrapped note after
  the complete formatted date.
- [x] The note renders in bibliography output only; a GB/T author-date
  citation for an annotated-date reference emits no note (verified
  end-to-end against the embedded style). The pinned corpus's eight
  author-date citation scenarios are not yet re-verified against this
  change via the oracle/workflow-test harness.
- [x] Year suffixes precede the annotation, for example
  `1947a（民国三十六年）`.
- [x] Full-width CJK delimiters come from the existing `csl26-k2kp`
  script-aware wrap renderer (`realize_wrap`), not a normalization pass
  added by this feature.
- [x] Legacy GB/T note-field input converts to `DateValue` without
  regressing copyright, printing, approximate, no-date, or scalar-date
  behavior.
- [x] GB/T bibliography output includes the annotations for the §7.5.4.1
  records — and, it turns out, three more already-annotated records
  elsewhere in the pinned corpus (`gbt7714.8.2.2:2`, `:8.12.3:1`,
  `:8.12.3:3`), all five verified against the real
  `tests/fixtures/test-items-library/gb-t-7714-2025.json` corpus across
  all three GB/T styles (`author-date`, `numeric`, `note`), not synthetic
  data.
- [x] Focused standard-derived tests
  (`crates/citum-engine/tests/date_annotations.rs`) and a registered
  citeproc-js divergence (`div-015`,
  `docs/adjudication/DIVERGENCE_REGISTER.md`) document why bare-year
  oracle parity is not conformant for these records.
- [ ] Generated schemas, coverage audits, `just pre-commit`, and PR checks
  pass before the feature is marked Active.

## Changelog

- v1.6 (2026-07-20): Per review feedback (PR #1068), enable `note-wrap`
  in `gb-t-7714-2025-numeric` and `gb-t-7714-2025-note`, not just
  `-author-date`. Verified against the real pinned corpus
  (`tests/fixtures/test-items-library/gb-t-7714-2025.json`), which turned
  out to already contain five annotated records — not just the two
  §7.5.4.1 examples — across `gbt7714.7.5.4.1:1`/`:2`, `gbt7714.8.2.2:2`,
  `gbt7714.8.12.3:1`, `gbt7714.8.12.3:3`. Added a Rust regression test
  against the real corpus (`crates/citum-engine/tests/date_annotations.rs`)
  and registered the expected citeproc-js divergence (`div-015`,
  `docs/adjudication/DIVERGENCE_REGISTER.md`). Confirmed
  `gb-t-7714-2025-numeric` is not in `core-quality-baseline.json`'s
  CI-gated style set, so no oracle-comparator masking code was needed.
- v1.5 (2026-07-20): Per review feedback, document the bare-note render
  extension point: `note-wrap` currently always applies one of
  `parentheses`/`brackets`/`quotes`, with no delimiter-free option
  (`1947 民国三十六年` rather than `1947（民国三十六年）`). Not
  implemented; a future `WrapPunctuation::None` variant would extend the
  general wrap vocabulary, not add note-specific machinery. No behavior
  change.
- v1.4 (2026-07-20): Status Draft → Active. Implements the feature:
  `EdtfString` renamed to `DateValue` in place (no new wrapper type, no
  per-field reshaping — see Public data type), `DateConfig.note_wrap`,
  bibliography-scoped rendering through the existing `realize_wrap`, legacy
  `issued-note-literal` conversion (`annotated_issued_from_legacy`,
  `crates/citum-schema-data/src/reference/conversion/mod.rs`), and GB/T
  author-date bibliography enablement. Remaining acceptance-criteria gaps:
  the pinned corpus's two real §7.5.4.1 records are not yet annotated with
  `note` input, and the eight author-date citation scenarios have not been
  re-verified through the oracle/workflow-test harness against this change.
- v1.3 (2026-07-20): Generalize `calendar-note` to a plain opaque `note`
  sub-field, applicable to any date, not a calendar-specific type. Move the
  render opt-in from `MultilingualConfig.calendar-note-wrap` to a flat
  `DateConfig.note-wrap`, alongside `era-labels` and `range-delimiter`. The
  `csl26-k2kp` script-aware wrap renderer this feature depends on has
  landed (completed); the feature is no longer blocked, only unimplemented.
- v1.2 (2026-07-19): Retarget the rendering prerequisite. The script-aware
  wrap renderer is now increment 1 of the punctuation realization layer
  (`docs/specs/PUNCTUATION_REALIZATION.md`, bean `csl26-k2kp`), which
  absorbs the retired draft bean `csl26-kneq`; this feature depends on that
  increment alone.
- v1.1 (2026-07-18): Revise the render opt-in after review found the original
  mechanism unbuildable. The note wrap now reuses the `wrap` value model
  as a single bibliography-scoped style option rather than a
  per-`TemplateDate` field, gated on a script-aware wrap renderer for
  full-width CJK delimiters. Document bibliography-only scoping, deliberate
  trailing placement (extensible via `position`), full-width-only legacy
  conversion, and a serialized-corpus round-trip criterion.
- v1.0 (2026-07-18): Initial reviewer draft.
