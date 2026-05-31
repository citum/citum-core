# Sorting Specification

**Status:** Active
**Date:** 2026-05-31
**Related:** [`EXPLICIT_DEFAULT_SORTING.md`](./EXPLICIT_DEFAULT_SORTING.md),
  [`UNICODE_BIBLIOGRAPHY_SORTING.md`](./UNICODE_BIBLIOGRAPHY_SORTING.md),
  [`MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`](./MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md),
  sorting sections of [`MULTILINGUAL.md`](./MULTILINGUAL.md)

## Purpose

Canonical end-to-end specification for bibliography and citation sorting in Citum.
Sorting predates the spec-driven-development policy, so this document captures shipped
behavior and design intent, and references narrower specs that extend or constrain it.

## Scope

**In scope:** bibliography sort resolution, citation-cluster sort policy, sort keys and
presets, collation, secondary/tiebreak rules, grouping interplay.

**Out of scope:** script/language partitioning (see
[`MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`](./MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md)),
transliteration-aware sort keys, per-entry sort overrides.

## Core Separation of Concerns

Following biblatex's design, Citum separates sorting into two independent concerns:

| Concern | Schema location | Who may set it |
|---|---|---|
| Bibliography ordering | `bibliography.sort` | Style author; falls back to processing-family default |
| Citation-cluster ordering | `citation.sort` | Style author only; no family default |

These two sort specifications are fully independent. A style may have a `citation.sort`
without any `bibliography.sort`, or vice versa.

## Bibliography Sort Resolution

Resolution is applied at bibliography-render time, in priority order:

1. Explicit `bibliography.sort` in the style YAML.
2. Processing-family default (`Processing::default_bibliography_sort()`).
3. No sort — preserve insertion order.

### Processing-Family Bibliography Defaults

| Processing class | Default bibliography sort preset |
|---|---|
| `author-date` | `author-date-title` |
| `note` | `author-title-date` |
| `label` | `author-date-title` |
| `numeric` | None (insertion order) |

These defaults exist because author-date and note families have strong conventional
ordering, while numeric styles depend on their own numbering logic.

**Implementation:** `crates/citum-schema-style/src/options/processing.rs` →
`Processing::default_bibliography_sort()` and `Processing::config()`.

## Citation-Cluster Sort Policy

Citation-cluster ordering is explicit-only in the current implementation:

- If `citation.sort` is present, apply it.
- Otherwise preserve citation input order.

No processing family provides an implicit citation-list sort. This mirrors
biblatex's `sortcites` opt-in philosophy.

## Sort Keys

Sort keys are defined by `SortKey` (non-exhaustive) in
`crates/citum-schema-style/src/options/processing.rs`:

| Key | Semantics |
|---|---|
| `Author` | Primary author name (family-first); falls back to editor, then title if no contributor |
| `Year` | Issued date year; year-bearing entries precede year-less entries |
| `Title` | Title text with locale article stripping (see `Locale::strip_sort_articles`) |
| `CitationNumber` | Numeric citation order (used internally; always equal in sort comparisons) |

Each key has an `ascending` flag (default `true`).

## Sort Presets

Named `SortPreset` values resolve to fixed `SortKey` chains:

| Preset | Key chain |
|---|---|
| `author-date-title` | `Author → Year → Title` |
| `author-title-date` | `Author → Title → Year` |

Styles may also supply a custom `SortSpec` template instead of a named preset.

## Collation

All text comparisons (author, title keys) use a locale-aware `TextCollator`
(`crates/citum-engine/src/sort_support.rs`), backed by ICU4X when the `icu`
feature is enabled.

Configuration:
- **Strength:** Secondary — base letters and diacritics distinguished; case is not.
- **Case level:** Off — case-insensitive via collator configuration, not lowercasing.
- **Alternate handling:** Shifted — punctuation and whitespace ignorable at primary/secondary
  levels (leading "al-", "O'", etc. do not break alphabetical ordering).
- **Locale fallback:** Progressively strips subtags (`de-DE-x` → `de-DE` → `de`) until a
  recognized locale is found; falls back to `en-US`.

Full collation semantics are specified in
[`UNICODE_BIBLIOGRAPHY_SORTING.md`](./UNICODE_BIBLIOGRAPHY_SORTING.md).

## Deterministic Tiebreaking

When all sort-key comparisons produce `Equal`, entries are ordered by citation-key
string comparison (`id.0.as_str()`). Entries without an ID sort last. The underlying
sort is stable, so entries that are collator-equal through all steps retain their original
input order if their IDs are also equal.

## Grouping Interplay

- Numeric citation-number initialization and year-suffix/disambiguation ordering both
  consume the resolved bibliography sort; they must be applied after sort resolution.
- Grouped bibliographies (`bibliography.groups`) apply their own per-group sort independently.
  Partition-aware sorting (`sort-partitioning`) runs as a pre-pass before the normal key
  chain; see [`MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`](./MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md).

## Key Implementation Files

| File | Role |
|---|---|
| `crates/citum-engine/src/processor/sorting.rs` | `Sorter` struct; multi-key sort dispatch |
| `crates/citum-engine/src/sort_support.rs` | `TextCollator`, `author_sort_key_opt`, `title_sort_key` |
| `crates/citum-engine/src/grouping/sorting.rs` | Grouped bibliography sort integration |
| `crates/citum-engine/src/sort_partitioning.rs` | Script/language partition pre-pass |
| `crates/citum-schema-style/src/options/processing.rs` | `SortKey`, `SortSpec`, `SortEntry`, `SortPreset`, `Processing::default_bibliography_sort()` |

## Test Anchor

`crates/citum-engine/tests/sort_oracle.rs` — end-to-end bibliography and citation sort
behavior. Bibliography sort sub-module: `crates/citum-engine/tests/bibliography.rs::sorting`.

## Open Work

- `EXPLICIT_DEFAULT_SORTING.md` tracks the implementation of `Processing::default_citation_sort_policy()`
  and the formalization of `CitationSortPolicy::ExplicitOnly` as a public API (currently
  behavior-only, not an enum value).
- Per-script partitioning (`sort-partitioning`) acceptance criteria are tracked in
  `MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`.

## Changelog

- 2026-05-31: Initial version — documents shipped behavior; references narrow sub-specs.
