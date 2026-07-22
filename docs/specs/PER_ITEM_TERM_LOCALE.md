# Per-Item Term Localization Specification

**Status:** Active
**Version:** 1.1
**Date:** 2026-07-18
**Related:** [`MULTILINGUAL.md`](./MULTILINGUAL.md) §3.3–3.4,
[`LOCALE_MESSAGES.md`](./LOCALE_MESSAGES.md),
[2026-07-18 multilingual architecture audit](../architecture/audits/2026-07-18_MULTILINGUAL_ARCHITECTURE_AUDIT.md)
§2(g); bean `csl26-838l`

## Purpose

Let a style render locale-sensitive terms in each *item's* language rather
than the style's, without swapping template structure. The motivating case:
a German source cited in an English-locale Chicago-style document should
read "hrsg. von" while its English neighbors read "edited by" — the
behavior biblatex provides with `autolang` and CSL-M approximates with
per-item `default-locale`. Today Citum can only change term language
through `citation.locales[]` / `bibliography.locales[]` branches, which
replace the entire template to change the language of a handful of words.

## Scope

**In scope:** a single opt-in (`options.multilingual.term-locale`), the set
of lookups that switch with it, the locale resolution and fallback chain,
precedence against locale-scoped layout branches, and the engine's
multi-locale threading requirement.

**Explicitly out of scope:** per-category granularity (switching roles but
not dates, etc. — recorded as a possible future field, not designed here);
any change to `grammar-options` typography scoping (quote characters,
collision policy, note punctuation stay with the style locale — see
Design §4); locale-scoped layout branch semantics (`MULTILINGUAL.md` §3.4,
unchanged); data-language-driven *content* selection
(`title-mode`/`name-mode`, unchanged); sorting, collation, and
disambiguation, which never consult term locale.

## Design

### 1. Opt-in

```yaml
options:
  multilingual:
    term-locale: item    # style | item; default style
```

`term-locale` is a new optional field on `MultilingualConfig`, available at
the same three scopes as the rest of that config — global `Config`,
`CitationOptions`, and `BibliographyOptions` — with the existing merge
precedence. The default `style` is today's behavior, byte for byte. A
style can therefore enable item-language terms in the bibliography while
keeping citations in the style locale, or vice versa:

```yaml
bibliography:
  options:
    multilingual:
      term-locale: item
```

### 2. What switches

With `term-locale: item`, the following lookups resolve against the
**effective item language's locale** instead of the style locale, per
rendered item:

- role labels and verb phrases (`roles:` / `role.*` messages) — "Hrsg.",
  "hrsg. von";
- locator labels (`locators:` / `*.label` messages) — "S." vs "p.";
- terms and phrase patterns (`terms:` / `term.*`, `pattern.*` messages) —
  "In:", "and", "et al.", accessed/retrieved phrases;
- date rendering: month and season names, `pattern.date-*` messages, era
  terms, and the locale date-format patterns.

The effective item language is the same per-item resolution already used
for locale-scoped layout selection and script detection (`MULTILINGUAL.md`
§3.2a/§3.4), including `field-languages` overrides where a field-scoped
language applies.

Everything the item supplies as *data* (names, titles, `calendar-note`) is
untouched — this spec governs engine-supplied words only.

### 3. Locale resolution and fallback

For an item with effective language `L`, the term locale resolves:

1. a loaded locale exactly matching `L` (case-insensitive BCP 47 match);
2. a loaded locale matching `L`'s primary language subtag;
3. the style locale.

"Loaded" means the embedded locale set plus any user-supplied locale
files. Resolution never fails and never selects a locale that is not
loaded — an unavailable item locale falls back to the style locale, the
same rule locale-scoped layout branches follow (`MULTILINGUAL.md` §3.4).
The fallback should emit the engine's standard silent-fallback diagnostic
so a missing locale is discoverable rather than invisible.

Untagged items resolve to the style locale (positive-evidence rule: no
language evidence, no switch).

### 4. What does not switch: the typography split

`grammar-options` typography — quote characters, `punctuation-in-quote`,
collision policy, note punctuation, page-range and subtitle delimiters —
**stays with the style locale** under `term-locale: item`. Terms are the
*item* speaking; typography is the *document* speaking. The German item in
an English document gets "hrsg. von" set inside English quote and
punctuation conventions, which is standard editorial practice for
English-language publications.

This is a deliberate v1 boundary, not an oversight: biblatex under
`autolang` does switch some intra-entry punctuation behavior with babel,
and a future revision may allow an explicit opt-in for item-locale
typography once the punctuation realization layer
([`PUNCTUATION_REALIZATION.md`](./PUNCTUATION_REALIZATION.md)) gives
typography a principled per-item selector. V1 keeps the split hard.

One narrower consequence of the same split: the *locale identifier* used to
gate English-only casing transforms (`text-case` on a switched term) is the
style locale, not the item locale, even though the term text itself is the
item's. A German role label rendered under an en-US style is casing-gated as
if it were English text. This is a byproduct of keeping typography-adjacent
behavior on the style side rather than a separately designed feature; it may
be revisited alongside the typography selector above.

### 5. Precedence

A matched `citation.locales[]` or `bibliography.locales[]` branch is
authoritative: it already selects both template structure and rendering
locale, and `term-locale` contributes nothing to items that match a
branch. `term-locale: item` applies on the default-branch and top-level
`template` paths — precisely the styles that share one structure across
languages and only need the words to follow the item.

This decouples the two axes: locale-scoped branches remain the tool for
*structurally* different per-language layouts; `term-locale` is the
lightweight tool for term language alone.

### 6. Rendering-only guarantee

Term locale affects rendered words only. Sorting keys, collation locale
(`options.sorting.locale`), partition detection, author-date grouping,
year-suffix assignment, and disambiguation comparisons are computed
exactly as under `term-locale: style`. Two otherwise-identical references
in different languages do not become distinct, and no reference reorders,
because its terms render in another language.

## Implementation Notes

Non-normative pointers:

- Schema: `term-locale` on `MultilingualConfig`
  (`crates/citum-schema-style/src/options/multilingual.rs`), a two-variant
  enum with `Style` as default; regenerate schemas (`just schema-gen`) in
  the same commit.
- The engine threads one `&Locale` through processing (e.g.
  `processor/citation.rs`, `processor/matching.rs`,
  `processor/disambiguation.rs`). `item` mode requires the loaded-locale
  registry to be reachable at term-resolution sites so the per-item locale
  can be selected there — the same machinery locale-scoped layout branches
  already rely on for their branch-locale rendering; reuse that loading
  and fallback path rather than adding a second registry.
- Term lookups flow MF2-first with legacy fallback (engine `CLAUDE.md`:
  `messages:` consulted before `terms:`/`roles:`/`locators:`); per-item
  selection happens at the locale-choice level, above that mechanism,
  which is unchanged.
- Embedded locale coverage directly bounds this feature's usefulness
  (`csl26-tfi8`, `csl26-itri`): a ja-JP item cannot render ja terms until
  a ja-JP locale exists. Fallback semantics make that safe, not silent —
  see the diagnostic in Design §3.

## Acceptance Criteria

- [x] `term-locale` absent or `style`: byte-identical output for all
  existing styles and fixtures.
- [x] With `term-locale: item` in bibliography options, a German-language
  reference in an en-US-locale style renders German role labels and terms
  ("hrsg. von", "In:") while English references are unchanged, in one
  bibliography.
- [x] Citations and bibliography scope independently (item terms in one,
  style terms in the other).
- [x] Month names and date patterns follow the item locale under `item`;
  quote characters and collision policy follow the style locale in the
  same entry.
- [x] An item whose language has no loaded locale falls back to style
  locale terms and emits the standard silent-fallback diagnostic.
- [x] Untagged items render style-locale terms.
- [x] Items matched by a `citation.locales[]`/`bibliography.locales[]`
  branch render with the branch locale regardless of `term-locale`.
- [x] Sort order, grouping, year suffixes, and disambiguation are
  identical under `style` and `item` for the same input.
- [x] Generated schemas include `term-locale`; all new public Rust items
  are documented.

## Changelog

- v1.0 (2026-07-18): Initial draft, from the 2026-07-18 multilingual
  architecture audit §2(g). Defines the opt-in, the switching term set,
  fallback chain, the terms/typography split, and precedence against
  locale-scoped layout branches.
- v1.1 (2026-07-22): Implemented and activated (`csl26-838l`). Added the §4
  note on the casing-gate/style-id boundary; all acceptance criteria met.
