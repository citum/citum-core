# Multilingual Sorting Specification

**Status:** Draft
**Version:** 1.0
**Date:** 2026-07-08
**Related:** beans `csl26-xz2t`, `csl26-6rjq`;
  [`SORTING.md`](./SORTING.md),
  [`UNICODE_BIBLIOGRAPHY_SORTING.md`](./UNICODE_BIBLIOGRAPHY_SORTING.md),
  [`MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`](./MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md),
  [`MULTILINGUAL.md`](./MULTILINGUAL.md) §4

## Purpose

Define the public schema options for multilingual bibliography sort policy and
the transliteration-aware sort keys they consume, so that non-Latin names and
titles (Arabic, Cyrillic, CJK) can optionally sort under romanized forms while
rendering unchanged in the original script. This is the expectation in library
and archival contexts, where a Cyrillic Толстой is filed under "T" (ALA-LC
*Tolstoy*), not under the collation position of "Т".

This spec jointly resolves beans `csl26-xz2t` (schema options) and
`csl26-6rjq` (transliteration-aware sort keys): the two are one design because
the data-model answer determines the schema surface.

## Scope

**In scope:** the `options.sorting` schema block (style level, with
bibliography-level override), the `sort-as` reference-data fields, the sort-key
resolution chain for each multilingual sorting mode, and the relationship of
`per-script` mode to bibliography sort partitioning.

**Explicitly out of scope:** generated (engine-computed) transliteration — that
is Phase 3, feature-gated, and defined here only at the contract level; any
change to how names or titles *render* (display of romanized forms remains
governed by `options.multilingual`, see [`MULTILINGUAL.md`](./MULTILINGUAL.md)
§2–3); per-entry sort overrides such as a whole-entry biblatex `sortkey`;
changes to the core collator configuration.

## Layering on Existing Specs

Multilingual sorting is an **optional layer** on top of two established
mechanisms. Nothing in this spec changes their defaults:

1. **Unicode collation baseline.** Locale-tailored single-collator UCA/CLDR
   sorting, as specified in
   [`UNICODE_BIBLIOGRAPHY_SORTING.md`](./UNICODE_BIBLIOGRAPHY_SORTING.md),
   remains the base comparison for all text sort keys, including every
   fallback position in this spec.
2. **Per-script partitioning.** Partition detection, ordering, and section
   rendering are governed by
   [`MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`](./MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md)
   (`bibliography.options.sort-partitioning`). This spec adds only a shorthand
   that expands to that mechanism (§Design 3).

## Design

### 1. Schema: the `sorting` options block

A new typed options block, available at style level (`options.sorting`) and as
a bibliography-scoped override (`bibliography.options.sorting`) following the
unified scoped-options model
([`UNIFIED_SCOPED_OPTIONS.md`](./UNIFIED_SCOPED_OPTIONS.md)):

```yaml
options:
  sorting:
    locale: auto            # auto | <bcp47>, default auto
    multilingual: uniform   # uniform | romanized | per-script, default uniform
```

- **`locale`** — the collator locale used for text sort comparisons.
  - `auto` (default): the effective bibliography locale, resolved through the
    existing fallback chain in `UNICODE_BIBLIOGRAPHY_SORTING.md` §Collation
    Policy: parse the BCP 47 identifier; if unrecognized, progressively strip
    subtags (`de-DE-foo` → `de-DE` → `de`); if nothing resolves, fall back to
    `en-US`. That final fallback is a **stability guarantee** (reproducible
    order everywhere), not a linguistic-correctness guarantee for scripts with
    their own tailored ordering.
  - An explicit BCP 47 tag pins the collator locale independently of the
    style/bibliography locale (e.g. sort a German-language bibliography with
    Swedish collation). The same fallback chain applies to the explicit tag.
- **`multilingual`** — the multilingual sort mode. A **single value**;
  modes do not combine (see §Design 3 for how romanized-within-partitions is
  expressed instead).

`multilingual: uniform` is the default and is bit-for-bit today's behavior:
one collator, one pass, no consultation of `sort-as` even when present.
Existing styles and data are unaffected unless a style opts in.

### 2. Data model: `sort-as` keys

Reference data may supply an explicit romanized (or otherwise
sort-normalized) key alongside the original text. Prior art is biblatex's
`sortname` / `sorttitle` / `sortkey` fields, which serve exactly this
role: a hidden key that controls filing order while the displayed text stays
untouched.

Two attachment points:

- **`sort-as` on `MultilingualComplex`** (multilingual titles and multilingual
  name *parts*, since `StructuredName.given`/`family` are `MultilingualString`
  and `Title::Multilingual` wraps `MultilingualComplex`):

  ```yaml
  title:
    original: "Война и мир"
    lang: ru
    sort-as: "Voĭna i mir"
    transliterations:
      ru-Latn-alalc97: "Voĭna i mir"
  ```

- **`sort-as` on `MultilingualName`** (holistic whole-name key, the analogue
  of biblatex `sortname`):

  ```yaml
  author:
    original:
      family: "Толстой"
      given: "Лев"
    lang: ru
    sort-as: "Tolstoy"
    transliterations:
      ru-Latn-alalc97:
        family: "Tolstoĭ"
        given: "Lev"
  ```

Semantics:

- `sort-as` is **purely structural**. It is never rendered, in any mode, under
  any display configuration. Rendering of romanized forms is a separate
  concern controlled by `options.multilingual` (title-mode / name-mode /
  patterns).
- `sort-as` is free text supplied by the data producer; Citum does not
  validate that it is a transliteration of the original, and it may
  legitimately differ from every `transliterations` entry (e.g. a filing form
  that drops an article or particle).
- When both attachment points are present for a name, the holistic
  `MultilingualName.sort-as` wins over a part-level key, mirroring biblatex
  `sortname` semantics (the whole-name key overrides per-part derivation).
- `sort-as` values are compared with the same collator as every other sort
  key (same locale, same normalization); they are keys into the existing
  comparison machinery, not a parallel ordering system.

### 3. Behavior by mode

#### `uniform`

The single-collator pass specified in `UNICODE_BIBLIOGRAPHY_SORTING.md`, using
`sorting.locale` as the collator locale. No partitioning is implied. `sort-as`
is ignored even when present — this guarantees that adding `sort-as` data to a
reference library never changes output for styles that have not opted in.

#### `romanized`

Sort keys for contributor names and titles resolve through a three-step
chain, per field:

1. **Explicit `sort-as`** on the relevant structure (holistic name key first,
   then part-level key), when present and non-empty.
2. **Matched transliteration** from the existing `transliterations` map,
   selected by the matching strategy already defined in
   [`MULTILINGUAL.md`](./MULTILINGUAL.md) §1.3 and used by rendering: exact
   BCP 47 tag (honoring `options.multilingual.preferred-transliteration`),
   then script-prefix match via `preferred-script` (default `Latn`).
3. **Original text** — identical to `uniform` for that field.

Every step feeds the same UCA/CLDR collator; the chain selects *which text*
is compared, never *how* it is compared. Step 2 implements the design intent
recorded in `MULTILINGUAL.md` §4.1 (sorting prefers the transliteration
variant even when the bibliography displays the original script) and keeps
sort order consistent with what romanized display modes actually render.

Scope policy: romanization applies only where the data supplies keys
(steps 1–2). There is **no global "romanize everything" mode** — references
without `sort-as` or matching transliterations behave exactly as under
`uniform`, and Latin-script references are unaffected throughout.
`romanized` does not itself imply partitioning.

#### `per-script`

A pure **shorthand** for the existing partitioning mechanism:

- When no explicit `bibliography.options.sort-partitioning` is configured,
  `multilingual: per-script` behaves exactly as if

  ```yaml
  bibliography:
    options:
      sort-partitioning:
        by: script
        mode: sort-only
  ```

  were set (partition order, headings, and all other semantics per
  `MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`).
- When an explicit `sort-partitioning` block **is** configured, it is
  authoritative and the shorthand contributes nothing. This is a defined
  precedence rule, not a conflict: the shorthand only fills an absent block.
- Within partitions, sort keys resolve with `uniform` semantics.

**Composition.** Because partitioning is a pre-pass orthogonal to key
construction, a style that wants romanized order *within* script partitions
does not combine mode values; it sets `multilingual: romanized` together with
an explicit `sort-partitioning` block:

```yaml
options:
  sorting:
    multilingual: romanized
bibliography:
  options:
    sort-partitioning:
      by: script
      mode: sort-only
      order: [Cyrl, Latn, Hani]
```

### 4. Policy decisions (canonical answers to csl26-6rjq)

1. **Which transliteration standard?** None is chosen globally. The primary
   mechanism is data-supplied keys (`sort-as`, or existing `transliterations`
   entries tagged with their standard, e.g. `ru-Latn-alalc97`). Generated
   transliteration is Phase 3: a per-script registry (ALA-LC for Cyrillic,
   Pinyin for Han), feature-gated, offered only where a deterministic Rust
   implementation exists; all other scripts stay on UCA collation.
2. **Is the sort key visible?** No. The sort key is a hidden romanized key;
   the bibliography renders original-script names and titles unchanged.
   Displaying romanized forms (alone or alongside the original) is controlled
   by multilingual *rendering* options, never by the sorting subsystem.
3. **Global or scoped?** Per-script/per-locale only, activated by
   `sorting.multilingual`. Unconfigured scripts and key-less references always
   degrade to plain UCA collation on the original text.

## Implementation Notes

Non-normative pointers for the implementing commits:

- Schema: `SortingConfig { locale, multilingual }` +
  `SortingMultilingualMode { Uniform, Romanized, PerScript }` in
  `crates/citum-schema-style/src/options/`, added to `Config` and
  `BibliographyOptions` with the existing merge machinery
  (`options/mod.rs` `merge`/`merge_shared_fields` patterns) and
  forward-compat `unknown_fields` conventions.
- Data: `sort-as` on `MultilingualComplex`
  (`crates/citum-schema-data/src/reference/types/common.rs`) and
  `MultilingualName` (`crates/citum-schema-data/src/reference/contributor.rs`),
  optional and skip-serialized when absent.
- Engine: thread the effective sorting config into
  `crates/citum-engine/src/sort_support.rs` (`author_sort_key_opt`,
  `title_sort_key`) and the `Sorter` paths (`processor/sorting.rs`,
  `grouping/sorting.rs`); reuse the existing `preferred_transliteration`
  resolution used by rendering for step 2 of the chain. `per-script`
  resolves to an effective `BibliographySortPartitioning` before the
  existing partition pre-pass.
- Schema generation (`just schema-gen`) in the same commit as any
  `citum-schema*` change.

### Phasing

- **Phase 1** — `sort-as` data fields; `options.sorting` with `uniform` and
  `romanized`; three-step chain in the engine. Self-contained; unblocks
  archival users without any transliteration engine.
- **Phase 2** — `per-script` shorthand wiring to `sort-partitioning`,
  including the precedence rule.
- **Phase 3** (separate bean, future PR) — transliteration registry and
  feature-gated generated `sort-as`, preserving hidden-key semantics.

Phases 1–2 ship in the PR that activates this spec; Phase 3 is deferred.

## Acceptance Criteria

- [ ] Existing styles and data produce byte-identical output when
      `options.sorting` is absent or `multilingual: uniform` (default no-op),
      including when references carry `sort-as` data.
- [ ] Under `romanized`, a Cyrillic name with `sort-as: "Tolstoy"` files under
      T among Latin peers while rendering in Cyrillic (archival ALA-LC case).
- [ ] Under `romanized`, a reference with no `sort-as` but a
      `ru-Latn-alalc97` transliteration sorts by that transliteration
      (step-2 fallback).
- [ ] Under `romanized`, a reference with neither key sorts identically to
      `uniform` (step-3 fallback), and Latin-script references are unaffected.
- [ ] Holistic `MultilingualName.sort-as` takes precedence over a part-level
      `MultilingualComplex.sort-as` on the same name.
- [ ] `sort-as` never appears in rendered citation or bibliography output in
      any mode/display combination.
- [ ] `multilingual: per-script` with no explicit `sort-partitioning` matches
      the output of `sort-partitioning: { by: script, mode: sort-only }`.
- [ ] An explicit `sort-partitioning` block wins over the `per-script`
      shorthand.
- [ ] `multilingual: romanized` composes with an explicit `sort-partitioning`
      block (romanized order within script partitions).
- [ ] `sorting.locale` accepts `auto` and explicit BCP 47 tags, both subject
      to the documented fallback chain.
- [ ] Generated schemas include `options.sorting`, the bibliography override,
      and the `sort-as` fields; all new public Rust items are documented.
- [ ] Mixed-script regression fixtures cover entries with and without
      `sort-as` in one bibliography.

## Changelog

- v1.0 (2026-07-08): Initial draft — joint design for beans `csl26-xz2t` and
  `csl26-6rjq`.
