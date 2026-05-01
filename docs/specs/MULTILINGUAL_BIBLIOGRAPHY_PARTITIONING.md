# Multilingual Bibliography Partitioning Specification

**Status:** Draft
**Date:** 2026-05-01
**Related:** `csl26-al0f`, `csl26-xz2t`, [`UNICODE_BIBLIOGRAPHY_SORTING.md`](./UNICODE_BIBLIOGRAPHY_SORTING.md)

## Purpose

Define style-controlled bibliography partitioning for multilingual bibliographies so entries can be ordered by script or language before normal author/date/title sorting, with optional visible sections for styles that need headings.

## Scope

In scope: flat partition-aware bibliography sorting, optional automatic bibliography sections, script partition detection from rendered sort text, language partitioning from effective item language, schema options, and regression tests. Out of scope: transliteration-aware sort keys, language-specific display names invented by the engine, and replacing explicit `bibliography.groups`.

## Design

Styles opt in with `bibliography.options.sort-partitioning`:

```yaml
bibliography:
  options:
    sort-partitioning:
      by: script
      mode: sort-only
      order: [Cyrl, Latn, Hani, Hira, Kana]
      headings:
        Cyrl: { literal: "Cyrillic" }
        Latn: { literal: "Latin" }
```

`by` selects the partition key source:

- `script`: derive an ISO 15924-style script code from the first significant character in the author sort key, then editor, then title.
- `language`: use the existing effective item language for the reference.

`mode` controls presentation:

- `sort-only`: render one flat bibliography, ordered by partition rank before the normal sort chain.
- `sections`: render automatic sections only for grouped bibliography output when no explicit `bibliography.groups` are present.
- `sort-and-sections`: apply both partition-aware sort order and automatic sections.

The `order` list is the only source of user-visible partition order. Values missing from `order` sort after configured partitions by their partition key. References with no detectable key sort last inside the partition tier. `headings` is optional and only used for section modes; the engine must not invent localized section labels.

Explicit `bibliography.groups` remains authoritative. When groups are present, automatic partition sections are disabled to avoid nested groups, but partition-aware sorting still applies inside each group unless the group declares its own sort.

## Implementation Notes

The motivating Zotero thread asks for Cyrillic entries first, then Latin, then Chinese/Japanese as a single continuous bibliography, not necessarily as formally headed sections: <https://forums.zotero.org/discussion/comment/280559/#Comment_280559>. That makes `sort-only` the default mode for parity with the primary user story.

Script detection should use Unicode Script properties through a direct engine dependency. Common scripts supported by the first implementation include `Latn`, `Cyrl`, `Arab`, `Hani`, `Hira`, `Kana`, and `Hang`; unsupported or inherited/common characters are skipped until a significant script character is found.

## Acceptance Criteria

- [ ] Existing styles preserve current single-collator sorting unless they opt in.
- [ ] `sort-only` partitions flat bibliographies before existing author/date/title sorting.
- [ ] `language` partitioning uses effective item language and configured order.
- [ ] `sections` renders partition headings only when grouped bibliography output is requested and no explicit groups exist.
- [ ] Explicit `bibliography.groups` disables automatic partition sections and preserves group-first semantics.
- [ ] Schema generation includes the new option and all public Rust items are documented.

## Changelog

- 2026-05-01: Initial draft.
