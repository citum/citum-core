# Citum Multilingual Support Design

**Status**: Active
**Authors**: @dstyleplan
**Date**: 2026-05-26

**Normative specs:** [`MULTILINGUAL_NAMES.md`](./MULTILINGUAL_NAMES.md),
[`MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`](./MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md),
[`MULTILINGUAL_SORTING.md`](./MULTILINGUAL_SORTING.md)

## Overview

This document outlines the architectural design for adding "elegant" multilingual support to Citum. The goal is to move away from procedural macros and toward a declarative, type-safe system that handles parallel metadata for high-fidelity citations.

## Core Principles

1.  **High-Fidelity Data**: Store original, transliterated, and translated versions of metadata fields side-by-side.
2.  **Declarative Style**: Styles request a specific "view" of the data (e.g., "transliterated [translated]") rather than implementing complex logic.
3.  **Graceful Degradation**: Simple use cases (monolingual data) must remain simple. The complexity of multilingual support should only be incurred when necessary.
4.  **Performance Check**: Heavy dependencies (like ICU4X for sorting) must be optional via feature flags.

## 1. Data Model

The core data model in `citum_schema` will be updated to support **Parallel Metadata**.

### 1.1 `Contributor` and `String` Fields

Currently, fields like `title` and `author` (via `Contributor`) primarily store single string values. We use a pattern to allow them to store complex objects without breaking the simple string ease-of-use.

**Schema (YAML) Examples:**

*Simple (Current Behavior):*
```yaml
title: "The Great Gatsby"
author: "Fitzgerald, F. Scott"
```

*Advanced (Multilingual Title):*
```yaml
title:
  original: "战争与和平"
  lang: "zh"
  transliterations:
    zh-Latn-pinyin: "Zhànzhēng yǔ Hépíng"
  translations:
    en: "War and Peace"
```

*Advanced (Multilingual Contributor):*
Names use a holistic multilingual approach where the entire name structure has parallel variants.

```yaml
author:
  original:
    family: "Tolstoy"
    given: "Leo"
  lang: "ru"
  transliterations:
    Latn:
      family: "Tolstoy"
      given: "Leo"
```

### 1.1a Field-Scoped Language Metadata

Entry-level `language` is not always enough.

Some records are genuinely mixed-language at the field level. A common case is an edited volume where:

- the chapter title is English
- the container book title is German
- the entry as a whole is still cataloged as German

For that case, Citum supports `field-languages` on the reference:

```yaml
title: "English Article"
language: de
field-languages:
  title: en
  parent-monograph.title: de
```

Interpretation:

- `language: de` remains the default language for the item
- `field-languages.title: en` overrides the language only for the chapter/article title
- `field-languages.parent-monograph.title: de` explicitly marks the container title as German

This is what "field-scoped language metadata" means in practice: language tags attached to specific bibliographic fields, not just to the whole entry.

Current engine-supported scopes:

- `title`
- `title-short`
- `parent-monograph.title`
- `parent-serial.title`

Unknown keys may be stored for forward compatibility, but current rendering logic ignores them.

### 1.2 Internal Representation

We use Serde's `untagged` enum feature to seamlessly support both formats. This model incorporates feedback that alternate fields need explicit language and script tagging.

```rust
// For Titles and simple strings
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum MultilingualString {
    Simple(String),
    Complex(MultilingualComplex),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct MultilingualComplex {
    pub original: String,
    pub lang: Option<LangID>,
    /// Transliterations/Transcriptions of the original text.
    /// Keys MUST be valid BCP 47 language tags including script and optional variant subtags.
    /// Script subtag specifies the writing system (Latn=Latin, Cyrl=Cyrillic, etc.).
    /// Variant subtag specifies the romanization method:
    ///   Japanese: "ja-Latn-hepburn" (Hepburn), "ja-Latn-kunrei" (Kunrei-shiki)
    ///   Chinese: "zh-Latn-pinyin" (Pinyin), "zh-Latn-wadegile" (Wade-Giles)
    ///   Russian: "ru-Latn-alalc97" (ALA-LC), "ru-Latn-bgn" (BGN/PCGN)
    /// Matching strategy: exact BCP 47 tag → script prefix → fallback to original.
    pub transliterations: HashMap<String, String>,
    pub translations: HashMap<LangID, String>,
}

// For Contributors
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum Contributor {
    SimpleName(SimpleName),
    StructuredName(StructuredName),
    Multilingual(MultilingualName), // Holistic parallel names
    ContributorList(ContributorList),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct MultilingualName {
    pub original: StructuredName,
    pub lang: Option<LangID>,
    pub transliterations: HashMap<String, StructuredName>,
    pub translations: HashMap<LangID, StructuredName>,
}
```

### 1.3 Transliteration Methods

Transliteration keys use BCP 47 language tags with script and variant subtags to specify the exact romanization system used.

**Common transliteration variants:**
- Japanese: `ja-Latn-hepburn` (Hepburn), `ja-Latn-kunrei` (Kunrei-shiki)
- Chinese: `zh-Latn-pinyin` (Pinyin), `zh-Latn-wadegile` (Wade-Giles)
- Russian: `ru-Latn-alalc97` (ALA-LC), `ru-Latn-bgn` (BGN/PCGN)

**Matching strategy:**
1. Exact BCP 47 tag match (e.g., "ja-Latn-hepburn")
2. Prefix match on script (e.g., "ja-Latn" matches any Latin transliteration)
3. Fallback to `original` field

Future: `preferred-transliteration` style option will allow explicit method selection.

## 2. Style Configuration

The `multilingual` key under `options` controls romanization and translation policy for
the style.  It accepts either a **preset name** or an **explicit configuration block**.

### 2.1 Preset names

Preset names describe the **rendering behavior**, not a specific style family.
They are the preferred way to configure multilingual rendering in embedded styles.

```yaml
options:
  multilingual: romanized-translated   # or: romanized-only, romanized-script-translated
```

| Preset | Title rendering | Name rendering | Typical use |
|---|---|---|---|
| `romanized-translated` | `romanized [translated]` (= `combined`) | romanized | APA, Chicago, MLA, Harvard, Vancouver, AMA, NLM, CSE |
| `romanized-only` | romanized only | romanized | IEEE and numeric styles |
| `romanized-script-translated` | `romanized original-script [translated]` | `romanized original-script` | Area-studies and East Asian studies house styles (e.g. CJKR, JAAS) |

All major English-language styles *require* romanized names and titles and *recommend*
a translation bracket for non-English titles.  Showing original-script text alongside
romanization is something these styles *allow* as a house option, not something they
mandate.  `romanized-script-translated` also sets `use-native-ordering: true` for
Han and Hangul scripts; for other scripts (Arabic, Cyrillic) the rendering pattern
is the same but script-specific ordering is unaffected.

### 2.2 Explicit configuration

For cases where no preset matches, a full block can be provided:

```yaml
options:
  multilingual:
    # Preferred view for titles.
    # Simple modes: primary | transliterated | translated | combined
    title-mode: combined          # combined = "romanized [translated]"

    # For three-way views use the pattern form:
    # title-mode:
    #   pattern:
    #     - view: transliterated
    #     - view: original-script
    #     - view: translated
    #       wrap: brackets       # none | brackets | parentheses

    # Preferred view for names.
    name-mode: transliterated

    # Preferred script for transliterations (e.g., "Latn", "Cyrl")
    preferred-script: "Latn"

    # Script-specific behavior
    scripts:
      cjk:
        use-native-ordering: true # FamilyGiven for CJK
        delimiter: ""            # No space between Family/Given
      latin:
        punctuation: latin       # Remap this script's rendered delimiters to Latin form
```

### 2.3 Pattern mode

`Pattern` is an ordered list of view segments joined by spaces.  It is used for styles
like Chicago (`romanized original-script [translation]`) or MLA (`original-script [translation]`)
that combine more than two views.  Segments whose resolved text is empty or identical to the
previous segment are silently skipped (dedup), so missing transliterations do not produce
duplicate text.

### 2.4 CSL-M localized layouts

CSL-M may provide an ordered series of `<layout locale="…">` branches followed
by one unscoped fallback layout. Migration preserves that order as `locales`
on the citation or bibliography section:

```yaml
citation:
  locales:
    - locale: [en, en-US]
      template:
        - title: primary
    - locale: [zh-CN]
      template:
        - contributor: author
    - default: true
      template:
        - identifier: cstr
```

Resolution tries the exact BCP 47 tag, then its primary language, then the
explicit default branch. A selected branch also establishes the rendering
locale for localized terms and dates. Conventional CSL with one unscoped
layout continues to use the ordinary `template` field.

The migration is intentionally narrow. Locale-specific layout wrappers,
citation-position overrides, or type-variant structures cannot be represented
independently inside a localized template and produce the stable
`unsupported-localized-layout-shape` migration diagnostic. Shared type
variants remain valid because Citum applies them after selecting the localized
branch while retaining its rendering locale.

## 3. Processor Logic

### 3.1 Value Resolution

For each multilingual field, the requested view (from `title-mode` / `name-mode`
or the preset) selects which variant renders:

- `primary` — the `original` text, unchanged.
- `transliterated` — the transliteration selected by the §1.3 matching strategy
  (exact BCP 47 tag → script prefix via `preferred-script` → fallback to
  `original`).
- `translated` — the translation matching the style locale's base language;
  falls back to `original` when no translation exists.
- `combined` — the transliterated view followed by the translated view in
  brackets (`romanized [translated]`); when no transliteration exists, the
  original takes its place. The bracket segment is dropped when the translation
  is missing or identical to the first segment.
- `pattern` — the ordered segment list defined in §2.3.

Simple (non-multilingual) values resolve to themselves under every mode.

### 3.2 Script-Aware Name Rendering

For contributors, the processor inspects the rendered given/family name parts to determine the active script, then applies script-specific ordering and separators from `options.multilingual.scripts`.

**Detection** uses the `unicode_script` crate, which covers all Unicode planes including CJK Unified Ideographs Extension B+ (U+20000 and above). Common or inherited punctuation does not force a script match. The supported script key set and matching order (exact key → ISO 15924 alias → group key → `cjk` umbrella) is specified in [`MULTILINGUAL_NAMES.md`](./MULTILINGUAL_NAMES.md).

**Ordering**: `use-native-ordering: true` on a matched script config renders family-first when the template has not explicitly requested another order. Template-level `name-order` and contributor sort settings remain authoritative and override `use-native-ordering`.

**Separators**: `delimiter` joins visible name parts in non-inverted rendering. `sort-separator` joins family and given when the name is inverted; it is a distinct field and is overridden by any component-level `sort-separator` attribute. The style config block example at the top of §2 shows `cjk: delimiter: ""` (no inter-part space) — per-script keys such as `katakana` can add a `・` delimiter and `、` sort-separator for that specific script.

Mixed-script names (e.g., family in Han, given in Katakana) collapse to the `cjk` umbrella unless the mix is purely Hiragana + Katakana, which resolves to `kana`. Dominance-based selection for other mixes is deferred.

If no matching script config exists, existing behavior is preserved: non-inverted names use spaces and inverted names use the normal contributor `sort-separator` fallback.

### 3.2a Script-Aware Punctuation

**Status: Active.**

Some bilingual styles hardcode one script's delimiter conventions for every item, regardless
of the individual reference's own language. GB/T 7714 is the motivating case: the style's
CJK-facing template punctuation (full-width `：，（）`) is also used, unmodified, for
Latin-script references — producing entries like `New York：Bantam Dell，1988` where GB/T
practice for a Latin-script reference is `New York: Bantam Dell, 1988`. This is a
verification-proxy gap, not a template bug: citeproc-js reproduces the same hardcoded
full-width punctuation, so byte-parity against citeproc-js does not catch it.

`options.multilingual.scripts.<script>.punctuation` is **not** a general punctuation policy
layer — it is a fixed, narrowly-scoped delimiter remap, opted into per script, for a single
known character set (the table below). It lives in the same `scripts.<script>` map that already
holds `delimiter`, `sort-separator`, and `use-native-ordering` for name rendering ([§3.2](#32-script-aware-name-rendering)),
following that section's existing per-script config pattern rather than introducing a new one:

```yaml
options:
  multilingual:
    scripts:
      latin:
        punctuation: latin   # omit the key (or set full-width) for no remap — the default
```

**Governing rule: absent or unrecognized evidence never triggers a remap.** Only `latin` is
implemented, and it only fires for items with positive evidence of a Latin-script effective
language — see "Script detection" below. `full-width` is not an independent behavior; it is an
explicit, documented alias for "no remap," identical to omitting the key. No other script value
is currently recognized.

`punctuation: latin` maps this script's rendered CJK full-width delimiters to Latin half-width
equivalents:

| Full-width | Latin |
|---|---|
| `：` (U+FF1A) | `: ` |
| `，` (U+FF0C) | `, ` |
| `（` (U+FF08) | `(` |
| `）` (U+FF09) | `)` |

This is exactly four characters, chosen because they are GB/T 7714's own template delimiters —
not a claim that these characters are universally interchangeable punctuation. The `（）`→`()`
row in particular is justified by GB/T's own usage: the style already uses full-width
parentheses as its field-wrap convention (issue numbers, citation numbers, the author-date
citation wrap), so remapping them is continuing that same convention in Latin form, not
asserting that parenthetical marks are style-neutral in general. A style that uses `（）` for
some other, non-delimiter purpose would need its own review before opting in here. The mapping
never runs in the other direction — a style's literal full-width components are untouched for
items whose script does not opt in.

*Implementation note:* a doubled space produced by the mapping (e.g. adjacent `：` and a
pre-existing space) is collapsed to one. This is processor-internal cleanup, not a separate
public guarantee.

**Script detection** reuses the effective-item-language resolution already used for
locale-scoped layout selection ([§3.4](#34-locale-selected-citation-and-bibliography-layouts)):
an explicit BCP 47 script subtag wins (`zh-Latn`, `ja-Hani`, …); otherwise a primary-language
lookup covers the common non-Latin scripts found in bilingual bibliographic corpora (CJK,
Cyrillic, Arabic, Hebrew, Greek, Devanagari). **An absent or unrecognized language tag is always
treated as not Latin** — the remap requires positive evidence of a Latin-script item, never
applying by default or by absence of contrary evidence. This is why Cyrillic items (e.g.
`ru-RU`) are intentionally left full-width for now rather than falling through to Latin: they
fail the positive-evidence test just as an untagged item would. Extending positive-evidence
coverage to other scripts (Cyrillic, Greek, Arabic, …) is a possible follow-up, scoped
separately — this feature does not anticipate or model their punctuation conventions today.

**Why three insertion points.** Full-width delimiters enter rendered output from three
independent places, and the remap has to run at each:

1. **Per-component rendering** (`render::component::render_component_with_format_and_renderer`)
   — a component's own `.value` (e.g. a joined name list) and its literal
   `prefix`/`suffix`/`wrap` from style YAML (e.g. `- number: issue prefix: （ suffix: ）`).
   This covers essentially all bibliography-entry punctuation, since bibliography components
   are otherwise fully assembled per-component.
2. **Citation-cluster wrap** (`render::citation::citation_to_string_with_format`) — the
   `wrap`/`prefix`/`suffix`/`delimiter` a citation section applies *around* its already-rendered
   components (e.g. GB/T author-date's own `citation.delimiter: ，`), which sits outside each
   component's own rendering.
3. **Citation-spec wrap** (`processor::citation::apply_spec_wrap_and_affixes`) — the outermost
   `citation.prefix`/`citation.suffix` wrap for the whole citation cluster (GB/T author-date's
   `citation: prefix: （ suffix: ）`), applied after all items in the citation are joined.

Each insertion point resolves the effective language independently: components carry their own
`item_language` already (threaded for locale-scoped rendering); the citation-cluster and
citation-spec wraps use the first item's reference in the citation as a stand-in for the whole
cluster (a reasonable approximation — compound citations mixing Latin- and CJK-script items
under one shared wrap are the unhandled edge case, left for follow-up if it proves to matter in
practice).

### 3.3 Locale Separation

The processor must distinguish between:
*   **Data Language**: The language of the source metadata (e.g., Russian).
*   **Style Locale**: The language of the citation style (e.g., English for "edited by").

Labels ("Ed.", "vol.") use the **Style Locale** unless a style explicitly
selects a locale-scoped layout under `citation.locales[]` or
`bibliography.locales[]`. A matched locale-scoped layout selects both its
template structure and its rendering locale. Data fields continue to use the
script determined by the **Data Language** and **Multilingual Mode**.

When `field-languages` is present, the processor should prefer the field-scoped language over the entry-level language for that specific field. This is how Citum can format a chapter title as English while formatting the containing book title as German in the same entry.

### 3.4 Locale-Selected Citation and Bibliography Layouts

`citation.locales[]` and `bibliography.locales[]` let a style swap the entire
template based on the reference's effective language. Each branch names the
locales it serves (`locale: [ja, zh, ko]`) or is marked `default: true`.

Resolution is deterministic:

1. Match a complete BCP 47 tag, case-insensitively.
2. If there is no exact match, match the primary language subtag.
3. Use the branch marked `default: true`.
4. Use the section's top-level `template` or `template-ref`.

A locale-scoped match uses the branch's declared locale for terms, labels,
dates, and other locale-sensitive formatting. A default branch or top-level
template continues to use the style locale. If the selected locale is not
available to the engine, rendering falls back to the style locale rather than
failing or silently selecting a different template.

Type variants compose with locale selection: a matching `type-variants` entry
may replace the resolved template, but it retains the locale selected by the
reference language. This permits one shared type-specific structure to render
with different term languages.

This is currently demonstrated by
`styles/experimental/locale-specific-bibliography-layouts.yaml`. These
selection and fallback rules are part of the active style schema contract.

## 4. Sorting & Transliteration

Sorting mixed scripts (e.g., Hanzi vs. Latin) requires Unicode Collation Algorithm (UCA) support.

### 4.1 Implementation

*   **Library**: Use `icu_collation` (ICU4X) for robust, locale-aware sorting.
*   **Logic**: Normatively specified in
    [`MULTILINGUAL_SORTING.md`](./MULTILINGUAL_SORTING.md). Transliteration-aware
    sorting is opt-in via `options.sorting.multilingual: romanized`; in that mode
    `author` and `title` sort keys resolve through a three-step chain — explicit
    `sort-as` key → transliteration variant matched per §1.3 → original text —
    even if the bibliography displays the `original` script. This ensures that
    "Толстой" (Cyrillic) sorts near "Tolstoy" (Latin) in an English bibliography.
    Under the default (`uniform`) mode, sorting always compares the original text.

### 4.2 Performance & Feature Flags

To avoid bloating the binary size for users who only need English/Simple citation support, all ICU4X dependencies will be gated.

```toml
[features]
default = []
multilingual = ["dep:icu_collation", "dep:icu_locid", "dep:icu_properties"]
```

## 5. Disambiguation

Citation disambiguation resolves surface-level ambiguity in written references for readers, not real-world identity verification.

### 5.1 Strategy

**Primary:** String matching on the displayed written form:
- If style displays `transliteration`, compare transliterated strings
- If style displays `original-script`, compare original script strings
- If style displays `translation`, compare translated strings

**Fallback:** If no exact match, use normalized comparison (Unicode NFC, case-folding)

### 5.2 PIDs Are Not Disambiguation Keys

Persistent identifiers (ORCID, DOI, ISBN) serve identity verification and linking purposes, but are NOT used for citation disambiguation. Two reasons:

1. **Scope mismatch**: PIDs identify entities globally, but disambiguation only needs to distinguish items within a single bibliography
2. **Display mismatch**: Readers see "Smith, J." vs "Smith, John" in text, not ORCIDs

PIDs remain valuable for metadata quality and cross-referencing, but disambiguation operates on rendered output strings.

## 6. Grouped Disambiguation

In complex multilingual bibliographies, a single global disambiguation scope can lead to confusing year suffixes. Citum enables localized disambiguation within bibliography groups.

### 6.1 Logic

- **Scope Control:** Use `disambiguate: locally` on a group to restart year suffix assignment.
- **Sorting Consistency:** Disambiguation keys follow the specific `sort` rules of the group (e.g., using `given-family` order for Vietnamese groups).
- **Multilingual Keys:** Disambiguator utilizes `Locale` to generate keys that are consistent with the scripts and name orders used within the group.
