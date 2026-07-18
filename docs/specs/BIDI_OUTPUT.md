# Bidirectional Output Handling Specification

**Status:** Draft
**Version:** 1.0
**Date:** 2026-07-18
**Related:** [2026-07-18 multilingual architecture audit](../architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md)
§2(d), [`MULTILINGUAL.md`](./MULTILINGUAL.md),
[`PUNCTUATION_REALIZATION.md`](./PUNCTUATION_REALIZATION.md); beans
`csl26-uzkj` (this spec), `csl26-30ga` (script resolution),
`csl26-5q59` (digit systems), `csl26-tfi8` (RTL-language locales)

## Purpose

Define how Citum keeps mixed-direction citations and bibliography entries
visually coherent. A bibliography entry for an Arabic or Hebrew source is
almost always mixed-direction — an RTL title next to a Latin-script DOI,
Western page numbers, and style punctuation — and without explicit
direction structure the Unicode Bidirectional Algorithm (UAX #9) reorders
the weak and neutral characters between runs, visually scrambling
punctuation, number ranges, and field order. The engine currently emits no
direction information of any kind; this spec adds it, per output format,
as an opt-in.

## Scope

**In scope:** inline direction isolation of rendered fields; the option
that enables it; the per-format realization (HTML/Djot vs the plain-text
family); direction detection and its evidence rules; entry-level direction
attributes where the format supports them; the fixture/linting caveat for
committed outputs containing control characters.

**Explicitly out of scope:** any reimplementation of the UBA (Citum adds
*structure*; display reordering remains the job of the browser, terminal,
or typesetter); paragraph/base direction of plain-text output (the
consuming application owns it); LaTeX and Typst realization (v1 documents
the limitation; both need engine-specific treatment — babel/polyglossia
macros, `text(dir:)` — that deserves its own increment); digit-system
conversion (`csl26-5q59`); RTL-aware line breaking or layout; sorting,
grouping, and disambiguation, which never consult direction.

## Background

Two distinct failure surfaces:

1. **Opposite-direction fields inside an entry.** An English-context
   bibliography entry for an Arabic work interleaves RTL runs (title,
   names) with LTR runs (DOI, "vol. 3", page ranges) joined by neutral
   punctuation. Under the UBA, the neutrals between runs take the
   surrounding direction unpredictably: page ranges display reversed,
   trailing punctuation jumps to the wrong end, and adjacent fields swap
   visual order. The standard remedy is to *isolate* each interpolated
   field so its internal direction cannot leak into the surroundings —
   `<bdi>` in HTML, FSI/PDI isolate controls in plain text.
2. **RTL entries in an LTR document (and vice versa).** A fully Arabic
   entry in an English bibliography should lay out right-to-left as a
   block. Block direction is representable only where the format has a
   block model (HTML `dir` attribute); plain text has no channel for it.

Isolates (U+2066 LRI, U+2067 RLI, U+2068 FSI, U+2069 PDI) are the
UAX #9-recommended mechanism and are used instead of the legacy
embedding/override controls, which are both deprecated in practice and the
subject of trojan-source security lint rules.

## Design

### 1. Option

```yaml
options:
  multilingual:
    bidi-isolation: none   # none | fields; default none
```

A new optional field on `MultilingualConfig`, available at its three
existing scopes (global `Config`, `CitationOptions`,
`BibliographyOptions`) with the existing merge precedence. `none` is
today's behavior, byte for byte — the opt-in preserves output parity with
citeproc-js and existing fixtures. The enum leaves room for a future
`entries` level without a schema break.

The option is a *style* concern with per-document override available
through the existing per-document configuration mechanism
([`PER_DOCUMENT_CONFIG_OVERRIDES.md`](./PER_DOCUMENT_CONFIG_OVERRIDES.md)):
an Arabic journal's style enables it unconditionally; a general-purpose
English style leaves it to the document.

### 2. Direction detection

Direction is a property of a rendered field's *content and metadata*,
resolved in this order:

1. **Metadata evidence:** the field's effective language/script (the same
   per-item, per-field resolution used elsewhere, `csl26-30ga`) maps to a
   direction via the script's UAX #24 property (Arab, Hebr, Syrc, Thaa →
   RTL; everything else → LTR).
2. **Content evidence:** when metadata is absent, the first strong
   directional character of the rendered field text decides (first-strong
   heuristic, the same rule FSI applies natively).
3. **No strong characters at all** (pure numbers/punctuation): the field
   has no direction of its own and is never isolated.

This extends the positive-evidence rule: strong RTL characters in the
content *are* positive evidence, so an untagged Hebrew title is still
protected. A field is isolated only when its resolved direction differs
from the **context direction** — the direction of the entry (or citation
cluster) it renders into, which itself resolves from the item's effective
language/script, falling back to the style locale's direction.

### 3. Per-format realization

Realized through the output-format abstraction, one strategy per format
family, applied at field boundaries during assembly (the same join points
the punctuation realization layer operates at):

| Format | Inline isolation | Entry direction |
|---|---|---|
| HTML | `<bdi>` around opposite-direction fields | `dir="rtl"`/`dir="ltr"` on the entry element when entry direction differs from bibliography direction |
| Djot | raw-HTML `<bdi>` span where the pipeline targets HTML; FSI/PDI otherwise | none (v1) |
| Plain, Markdown, Org | FSI (U+2068) … PDI (U+2069) around opposite-direction fields | none — base direction belongs to the consumer |
| LaTeX | **deferred** — requires babel/polyglossia or luabidi cooperation | deferred |
| Typst | **deferred** — `text(dir:)` wrapping, own increment | deferred |

Notes:

- HTML prefers `<bdi>` over control characters: it is self-documenting,
  inspectable, and cannot leak if markup is truncated. The existing
  semantic-span machinery (`semantic_with_attributes`) provides the
  insertion point.
- Plain-text isolates are emitted in balanced pairs only; an isolate is
  never opened without its PDI in the same field emission, so truncation
  by downstream consumers cannot strand an open isolate across entries.
- Deferred formats render exactly as today under `fields`; enabling the
  option must not emit half-supported direction syntax into LaTeX or
  Typst output.

### 4. What isolation does not do

- It never reorders anything itself; logical order in the output string is
  unchanged. Every byte of a `none`-mode string appears, in the same
  order, in the `fields`-mode string — interleaved with isolation
  structure only.
- It does not touch field-internal content: an RTL title containing an LTR
  acronym is left to the UBA, which handles single-field content
  correctly. Isolation is for the joints the *style* creates, where the
  UBA lacks the information that two adjacent runs are separate fields.
- It adds no isolation when every field in an entry shares the context
  direction — an all-Latin bibliography is byte-identical under `none`
  and `fields`.

### 5. Committed fixtures and control-character linting

Trojan-source lint rules reject bidi control characters in source trees —
demonstrated concretely on 2026-07-18, when alint v0.14.0's new
`oss-no-bidi-controls` rule failed this repository's CI over a single
stray U+200E in a test fixture. Expected-output fixtures for the
plain-text family under `fields` mode will contain FSI/PDI by design.
Implementation must therefore store those expectations in escaped form
(`\u{2068}` in Rust string literals, the equivalent `\uXXXX`
escapes in JSON) or grant the fixture
directory an explicit, documented lint exemption — decided at
implementation time, but the constraint is normative: **no raw isolate
controls committed to the tree without an exemption the hygiene tooling
recognizes.**

## Implementation Notes

Non-normative pointers:

- Schema: `bidi_isolation` on `MultilingualConfig`
  (`crates/citum-schema-style/src/options/multilingual.rs`); regenerate
  schemas (`just schema-gen`) in the same commit.
- Direction from script: derive from the `csl26-30ga` resolver's ISO
  15924 code; until then, the RTL primary-language subset already listed
  in `is_latin_script_language` (`ar`, `fa`, `ur`, `he`, `yi`) plus the
  `arab`/`hebr` script subtags gives an interim two-way classifier.
- First-strong scan: a single pass over the rendered field's chars using
  `unicode_bidi` or the bidi classes exposed via ICU4X properties — no
  new heavy dependency; gate with the existing `multilingual` feature if
  needed.
- Insertion points: field emission in `render/component.rs` and the
  entry-assembly path in `render/bibliography.rs`; HTML `<bdi>` through
  the format's semantic-span support, plain-text isolates through a small
  helper shared by the plain-family formats.
- The ar-AR embedded locale is currently minimal and there is no he
  locale (`csl26-itri`, `csl26-tfi8`); test data should use explicitly
  tagged references so this feature is testable before locale coverage
  catches up.

## Acceptance Criteria

- [ ] `bidi-isolation` absent or `none`: byte-identical output for all
  existing styles and fixtures.
- [ ] All-LTR entries are byte-identical under `none` and `fields`.
- [ ] Under `fields` in HTML, a Latin-script DOI field inside an
  Arabic-language entry is wrapped in `<bdi>`, and the entry element
  carries `dir="rtl"` in an LTR bibliography.
- [ ] Under `fields` in plain text, the same DOI field is wrapped in
  FSI/PDI, isolates are always balanced, and stripping all isolate
  characters reproduces the `none` output exactly.
- [ ] An untagged field whose text begins with strong RTL characters is
  isolated in an LTR context (content evidence); a pure-number field
  never is.
- [ ] LaTeX and Typst output is unchanged under `fields` (documented
  deferral).
- [ ] Sort order, grouping, year suffixes, and disambiguation are
  identical under `none` and `fields`.
- [ ] Committed fixtures containing isolates use escaped forms or a
  hygiene-recognized exemption; repository lint passes.
- [ ] Generated schemas include `bidi-isolation`; all new public Rust
  items are documented.

## Changelog

- v1.0 (2026-07-18): Initial draft, from the 2026-07-18 multilingual
  architecture audit §2(d). Defines the `bidi-isolation` opt-in,
  metadata/content evidence rules, per-format realization with `<bdi>`
  and FSI/PDI, deferrals for LaTeX/Typst, and the fixture linting
  constraint.
