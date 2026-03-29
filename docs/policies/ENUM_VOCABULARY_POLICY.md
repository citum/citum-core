# Enum and Controlled Vocabulary Policy

**Status:** Active Policy
**Version:** 1.0
**Date:** 2026-03-29
**Related:** TYPE_ADDITION_POLICY.md, csl26-ldgf (Genre bean)

## Purpose

This policy defines when a property should use a closed enum versus a free
string, how enum variants are named and serialized, and how unknown or future
values are handled. It also establishes localization-readiness constraints so
that canonical keys remain stable while future translation layers can attach
display labels.

## Scope

Applies to all types in `citum-schema-data` (reference data model). Does not
cover style-schema enums (template variables, contributor roles), which follow
their own evolution path.

## Core Principle: Identifiers, Not Labels

Every enum variant and every controlled-vocabulary string is a **stable
identifier**. It is not a user-facing label. English words are used for
readability, but they carry no linguistic commitment. A future localization
layer maps identifiers to display text; the stored value never changes.

```
canonical key (stored)  -->  locale map  -->  display label
"phd-thesis"            -->  en: "PhD thesis"
                             de: "Dissertation"
                             ja: "博士論文"
```

## Decision Criteria: Enum vs. String

Use the following three-factor test. A property should become a closed enum
when **all three** factors apply:

### 1. Bounded Domain

The set of valid values is finite and unlikely to grow beyond roughly 30
variants within the next two major versions.

- Yes: `MonographType` (Book, Report, Thesis, ...) -- bounded by scholarly publishing categories.
- No: `genre` -- open-ended descriptors like "PhD thesis", "Short film", "Assessment report" with no natural ceiling.

### 2. Style Discrimination

At least one citation style branches on the value to produce different
formatting. If no style ever inspects the value, it is display-only metadata
and a string suffices.

- Yes: `SerialType` -- styles distinguish journal articles from newspaper articles.
- No: `license` on Software -- purely informational, no style branches on it.

### 3. Serialization Stability

The value must round-trip identically across YAML, JSON, and CBOR without
normalization. Enums guarantee this by construction. Free strings require
documented conventions (case, punctuation) that producers may violate.

- Yes: type discriminators (`MonographType`, `SerialComponentType`) -- must match exactly.
- No: `format` on Dataset ("CSV", "csv", "Csv") -- producers already vary; enforcing an enum would break existing data.

### Decision Matrix

| Bounded | Style-discriminated | Round-trip critical | Decision |
|---|---|---|---|
| Yes | Yes | Yes | **Closed enum** with `#[non_exhaustive]` |
| Yes | Yes | No | Closed enum preferred; string acceptable |
| Yes | No | Yes | Enum if serialization bugs are likely; otherwise string |
| No | Yes | Yes | **Controlled vocabulary string** with documented canonical forms |
| No | No | * | Free string |

## Enum Naming Conventions

1. **Variant names** use `PascalCase` in Rust source (`AcademicJournal`, `EditedBook`).
2. **Serialized form** uses `kebab-case` via `#[serde(rename_all = "kebab-case")]` (`academic-journal`, `edited-book`). This is the canonical identifier stored in YAML, JSON, and CBOR.
3. **No embedded English prose.** Variant names describe the concept, not a display string. `PhDThesis` not `PhDThesisLabel`. `Preprint` not `PreprintArticle` (unless disambiguation is needed).
4. **Abbreviations** follow Rust conventions: two-letter abbreviations are uppercase (`PhD`), longer abbreviations are title-case (`Isbn`, `Doi`).

## Serialization Across Formats

| Format | Enum representation | Unknown value handling |
|---|---|---|
| YAML | kebab-case string: `type: academic-journal` | Deserialize rejects unknown variants with a clear error. Application layers MAY implement tolerant fallback/warn behavior. |
| JSON | Same kebab-case string: `"type": "academic-journal"` | Same as YAML (rejects unknown variants by default). |
| CBOR | Same string encoding (CBOR text string) | Same as YAML (rejects unknown variants by default). |

All three formats use identical string representations. No integer encoding
for enums in CBOR -- string stability takes priority over byte savings.

### Unknown Value Handling

Enums marked `#[non_exhaustive]` signal that new variants may be added. For
deserialization of unknown values:

1. **Strict mode** (current, default): reject unknown variants with a descriptive error naming the field and listing valid values. This is what derived `serde::Deserialize` provides.
2. **Proposed tolerant mode** (not yet implemented): in future, applications MAY implement a custom deserializer that maps unknown variants to a catch-all (e.g., `Other(String)`) and emits a warning, for forward compatibility when reading data from a newer schema version. This requires an explicit implementation — no built-in `Other(String)` variant exists today.

## Controlled Vocabulary Strings

For properties that remain `Option<String>` but benefit from a documented set
of expected values (e.g., `genre`, `medium`):

1. **Document canonical forms** in the field's doc comment and in a reference table in `docs/reference/`.
2. **Canonical forms use kebab-case** to match enum serialization conventions: `phd-thesis`, `short-film`, `television`.
3. **Matching should be case-insensitive** at the application layer. Producers should emit canonical forms; consumers should normalize before comparison. (Not all comparisons are currently case-insensitive; normalization is tracked in bean csl26-qqfa.)
4. **New canonical values** are added by PR to the reference table. No schema change is needed.
5. **Arbitrary values are permitted.** The canonical list is advisory, not exhaustive. Styles that branch on `genre` or `medium` should handle unrecognized values gracefully (typically by rendering the raw string).

## Adding New Enum Variants

1. Open a PR adding the variant to the Rust enum and updating the JSON Schema output.
2. The variant must include a `///` doc comment explaining what it represents.
3. Mark the enum `#[non_exhaustive]` if not already marked.
4. Update the serialization snapshot tests.
5. Update `docs/reference/` tables if applicable.
6. Bump the schema version: `patch` for a new optional variant, `minor` if styles need updating.

## Adding New Controlled Vocabulary Values

1. Open a PR updating the reference table in `docs/reference/`.
2. No Rust code change is required (the field is already `Option<String>`).
3. If the value is style-discriminated, add a fixture entry in `tests/fixtures/references-expanded.json`.

## Localization Readiness

### Constraints

1. **Canonical keys are immutable once released.** Renaming a kebab-case identifier is a breaking change. Deprecate and alias instead.
2. **Display labels live outside the data model.** The locale system (MF2 messages or a future `vocab-labels.yaml`) maps `(property, canonical-key, locale)` to display text. The reference data never contains display text for type/genre/medium values.
3. **Enum variants and controlled-vocabulary strings are the same namespace** from the localization layer's perspective. Whether `phd-thesis` is an enum variant or a documented string, the locale map treats it identically: `genre.phd-thesis = "PhD thesis"` (en), `genre.phd-thesis = "Dissertation"` (de).

### Future Localization Architecture (Non-normative)

A localization layer would provide:

```yaml
# locale/en/vocab.yaml
genre:
  phd-thesis: "PhD thesis"
  short-film: "Short film"
  assessment-report: "Assessment report"
medium:
  film: "Film"
  television: "Television"
  video-interview: "Video interview"
monograph-type:
  book: "Book"
  report: "Report"
  thesis: "Thesis"
```

The engine resolves display text at render time:
`canonical-key + locale --> display label`, falling back to the canonical key
itself (title-cased) when no translation exists.

### Backward Compatibility

- Adding a new canonical key: non-breaking. Old documents are unaffected. Old consumers ignore unknown keys.
- Renaming a canonical key: **breaking**. Existing YAML/JSON/CBOR documents contain the old key. Use deprecation aliases (`#[serde(alias = "old-name")]`) for a transition period of at least one major version.
- Removing a canonical key: **breaking**. Same transition-period requirement.

## Genre and Medium: Current Recommendation

### Genre

**Recommendation: keep as `Option<String>` with a documented controlled vocabulary.**

Rationale:
- The domain is open-ended. Fixture values include "PhD thesis", "Short film", "Assessment report" -- heterogeneous descriptors that cross reference-type boundaries.
- Styles that branch on `genre` (APA, Chicago, Springer) compare against known strings. An enum would not eliminate this comparison; it would just move the strings into variant names.
- Converting to an enum now risks frequent churn as new genre values surface during migration and style work.
- Localization can work equally well with documented string identifiers as with enum variants (see Localization Readiness above).

**Action items:**
1. Document canonical genre forms in `docs/reference/GENRE_AND_MEDIUM_VALUES.md`.
2. Normalize existing fixture values to kebab-case (`phd-thesis`, `short-film`, `assessment-report`).
3. Update the engine to compare case-insensitively.
4. Update bean csl26-ldgf to reflect this decision.

### Medium

**Recommendation: keep as `Option<String>` with a documented controlled vocabulary.**

Rationale:
- Same open-ended domain as genre. Values like "film", "Television", "Video interview" describe carrier/format, not a bounded set.
- Fewer styles branch on `medium` than on `genre`, reducing the urgency.
- Normalization to kebab-case and case-insensitive matching address the immediate consistency problem.

**Action items:**
1. Document canonical medium forms alongside genre in `docs/reference/GENRE_AND_MEDIUM_VALUES.md` (same file, separate section).
2. Normalize existing fixture values.

### Migration Path If Enums Are Needed Later

If the vocabulary stabilizes and enum conversion becomes desirable:

1. Define the enum with all documented canonical values plus `Other(String)`.
2. Implement `Deserialize` with a custom deserializer that maps known kebab-case strings to variants and unknown strings to `Other`.
3. Existing YAML/JSON/CBOR documents remain valid -- known values deserialize to the typed variant, unknown values land in `Other`.
4. This is a minor version bump (additive change, no breakage).

## types.rs Modularization Recommendation

At 897 lines, `types.rs` exceeds the project's 300-line guideline. Recommended split:

| New module | Contents | Approx lines |
|---|---|---|
| `types/common.rs` | `NumOrStr`, `MultilingualString`, `MultilingualComplex`, `Title`, `StructuredTitle`, `Subtitle`, `RefDate`, `ArchiveInfo`, `EprintInfo` | ~180 |
| `types/structural.rs` | `Monograph`, `MonographType`, `Collection`, `CollectionType`, `CollectionComponent`, `MonographComponentType`, `SerialComponent`, `SerialComponentType`, `Serial`, `SerialType`, `Parent`, `ParentReference` | ~280 |
| `types/legal.rs` | `LegalCase`, `Statute`, `Treaty`, `Hearing`, `Regulation`, `Brief` | ~200 |
| `types/specialized.rs` | `Classic`, `Patent`, `Dataset`, `Standard`, `Software` | ~200 |
| `types/mod.rs` | Re-exports | ~30 |

This preserves the public API (all types re-exported from `types::*`) while
keeping each file under 300 lines.

## Audit: Undocumented or Weakly Documented Items in types.rs

| Item | Line | Issue | Suggested fix |
|---|---|---|---|
| `RefID` | 19 | Type alias, no doc comment | Add `/// Unique identifier for a reference item.` |
| `LangID` | 20 | Type alias, no doc comment | Add `/// BCP 47 language tag (e.g., "en", "de", "ja").` |
| `FieldLanguageMap` | 21 | Type alias, no doc comment | Add `/// Maps field names to their language tags for multilingual references.` |
| `Monograph.id` | 190 | No doc comment | Add `/// Unique identifier for this reference.` |
| `Monograph.title` | 192 | No doc comment | Add `/// Title of the monographic work.` |
| `Monograph.author` | 195 | No doc comment | Add `/// Author(s) of the work.` |
| `Monograph.genre` | 224 | No doc comment | Add `/// Free-text genre descriptor (e.g., "phd-thesis"). See ENUM_VOCABULARY_POLICY.md.` |
| `Monograph.medium` | 225 | No doc comment | Add `/// Free-text medium descriptor (e.g., "film"). See ENUM_VOCABULARY_POLICY.md.` |
| `CollectionType` variants | 306-310 | No doc comments on any variant | Add one-line doc comment per variant |
| `MonographComponentType` variants | 354-356 | No doc comments on any variant | Add one-line doc comment per variant |
| `SerialComponentType` variants | 404-408 | No doc comments on any variant | Add one-line doc comment per variant |
| `MonographType::Manual` | 249 | No doc comment | Add `/// A technical manual or user guide.` |
| `MonographType::Post` | 253 | No doc comment | Add `/// A standalone post (e.g., social media, forum).` |
| `MonographType::PersonalCommunication` | 264 | No doc comment | Add `/// A letter, email, or other personal communication.` |
| `MonographType::Document` | 265 | No doc comment | Add `/// A generic standalone document.` |
| `Standard.status` | 847 | String field, should reference controlled vocabulary | Add doc noting canonical values: "published", "draft", "withdrawn" |
| `Software.platform` | 884 | String field, no guidance | Add doc noting this is a free-text descriptor |
| `Software.license` | 883 | String field, no guidance | Add doc noting SPDX identifiers are preferred |
| `Dataset.format` | 811 | String field, no guidance | Add doc noting IANA media types or common abbreviations |

## Changelog

**v1.0 (2026-03-29):**
- Initial policy establishing enum vs. string criteria
- Genre and Medium recommendation (keep as strings)
- Localization readiness constraints
- types.rs audit and modularization plan
