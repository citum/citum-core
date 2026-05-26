# Citum Multilingual Support Design

**Status**: Active
**Authors**: @dstyleplan
**Date**: 2026-05-26

**Normative specs:** [`MULTILINGUAL_NAMES.md`](./MULTILINGUAL_NAMES.md),
[`MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`](./MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md)

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

A new global configuration section `multilingual` will be added to the Citum style schema.

```yaml
options:
  multilingual:
    # Preferred view for titles.
    # The value is a mode key or a combined pattern such as "transliterated [translated]".
    # See the style schema for the authoritative grammar.
    title-mode: "transliterated [translated]"

    # Preferred view for names.
    # See the style schema for the authoritative value set.
    name-mode: "transliterated"

    # Preferred script for transliterations (e.g., "Latn", "Cyrl")
    preferred-script: "Latn"

    # Script-specific behavior
    scripts:
      cjk:
        use-native-ordering: true # FamilyGiven for CJK
        delimiter: ""            # No space between Family/Given
```

## 3. Processor Logic

### 3.1 Value Resolution

... [existing resolution logic] ...

### 3.2 Script-Aware Name Rendering

For contributors, the processor inspects the rendered given/family name parts to determine the active script, then applies script-specific ordering and separators from `options.multilingual.scripts`.

**Detection** uses the `unicode_script` crate, which covers all Unicode planes including CJK Unified Ideographs Extension B+ (U+20000 and above). Common or inherited punctuation does not force a script match. The supported script key set and matching order (exact key → ISO 15924 alias → group key → `cjk` umbrella) is specified in [`MULTILINGUAL_NAMES.md`](./MULTILINGUAL_NAMES.md).

**Ordering**: `use-native-ordering: true` on a matched script config renders family-first when the template has not explicitly requested another order. Template-level `name-order` and contributor sort settings remain authoritative and override `use-native-ordering`.

**Separators**: `delimiter` joins visible name parts in non-inverted rendering. `sort-separator` joins family and given when the name is inverted; it is a distinct field and is overridden by any component-level `sort-separator` attribute. The style config block example at the top of §2 shows `cjk: delimiter: ""` (no inter-part space) — per-script keys such as `katakana` can add a `・` delimiter and `、` sort-separator for that specific script.

Mixed-script names (e.g., family in Han, given in Katakana) collapse to the `cjk` umbrella unless the mix is purely Hiragana + Katakana, which resolves to `kana`. Dominance-based selection for other mixes is deferred.

If no matching script config exists, existing behavior is preserved: non-inverted names use spaces and inverted names use the normal contributor `sort-separator` fallback.

### 3.3 Locale Separation

The processor must distinguish between:
*   **Data Language**: The language of the source metadata (e.g., Russian).
*   **Style Locale**: The language of the citation style (e.g., English for "edited by").

Labels ("Ed.", "vol.") will always use the **Style Locale**. Data fields will use the script determined by the **Data Language** and **Multilingual Mode**.

When `field-languages` is present, the processor should prefer the field-scoped language over the entry-level language for that specific field. This is how Citum can format a chapter title as English while formatting the containing book title as German in the same entry.

## 4. Sorting & Transliteration

Sorting mixed scripts (e.g., Hanzi vs. Latin) requires Unicode Collation Algorithm (UCA) support.

### 4.1 Implementation

*   **Library**: Use `icu_collation` (ICU4X) for robust, locale-aware sorting.
*   **Logic**:
    *   If a sort key is `author` or `title`, the processor should prefer the `transliteration` variant if available, even if the bibliography displays the `original` script. This ensures that "Tolstoy" (Cyrillic) sorts near "Tolstoy" (Latin) in an English bibliography.

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
- If style displays `original`, compare original script strings
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
