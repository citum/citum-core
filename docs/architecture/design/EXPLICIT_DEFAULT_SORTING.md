# Explicit Default Sorting for Citation Classes

**Status:** Draft - revised for implementation
**Date:** 2026-02-28

## Summary

Citum should follow biblatex's separation of concerns:

- bibliography sorting is a bibliography concern
- multi-citation sorting is a citation concern
- processing families may provide bibliography defaults where the convention is strong
- explicit style settings always override family defaults

That means we should make bibliography defaults explicit for `author-date`,
`note`, and `label`, while keeping `citation.sort` explicit-only in Phase 1.

## Problem

Citation processing classes (`author-date`, `numeric`, `note`, `label`) imply
family-level bibliography conventions, but Citum currently encodes them
inconsistently:

- `Processing::config()` hardcodes an incomplete `author-date` fallback
  (`author + year`) and omits sort defaults for `note` and `label`.
- bibliography rendering, disambiguation, and numeric numbering consult
  explicit `bibliography.sort`, but otherwise fall back unevenly.
- real styles compensate in YAML by restating sorts such as
  `author-date-title`, which confirms the engine default is underspecified.

## Design Influence: biblatex

Biblatex handles this well because it separates:

- bibliography sort schemes (`sorting=...`)
- citation-list sorting (`sortcites`)
- per-context overrides (`refcontext`)

Citum should mirror that structure conceptually:

- `bibliography.sort` controls bibliography order
- `citation.sort` controls multi-cite ordering
- `processing` provides bibliography-family defaults only

We should not collapse those into one universal "class default sort."

## Canonical Bibliography Defaults

| Class | Default bibliography sort | Rationale |
|-------|----------------------------|-----------|
| `author-date` | `author-date-title` | Standard author-date bibliography ordering; title is the stable tiebreak |
| `numeric` | `None` | Numeric styles do not imply bibliography sorting; preserve insertion order unless explicit |
| `note` | `author-title-date` | Footnote styles with a bibliography are typically alphabetized by author, then title, then year |
| `label` | `author-date-title` | Alphabetic/alphanumeric families generally key off author-year-title ordering |

## Citation Sorting Policy

Citation sorting remains separate from bibliography sorting.

Phase 1 policy:

- if `citation.sort` is present, use it
- otherwise preserve the citation input order

No processing family gets an implicit citation-list sort in Phase 1.

## Recommendation

Implement this as an additive Rust-side change. Do not add new required YAML
fields.

### New Rust API

Add:

```rust
impl Processing {
    pub fn default_bibliography_sort(&self) -> Option<SortPreset>;
    pub fn default_citation_sort_policy(&self) -> CitationSortPolicy;
}
```

And:

```rust
pub enum CitationSortPolicy {
    ExplicitOnly,
}
```

## Implementation Steps

1. Add `Processing::default_bibliography_sort()` returning the canonical bibliography preset per processing variant.
2. Add `Processing::default_citation_sort_policy()` and define `CitationSortPolicy::ExplicitOnly`.
3. Update `Processing::config()` so bibliography-facing built-in defaults align with the family defaults:
   - `author-date` -> `author-date-title`
   - `note` -> `author-title-date`
   - `label` -> `author-date-title`
   - `numeric` -> none
4. Update bibliography sort resolution in `citum-engine`:
   - explicit `bibliography.sort`
   - otherwise `processing.default_bibliography_sort()`
   - otherwise preserve insertion order
5. Use the resolved bibliography sort for:
   - bibliography rendering
   - numeric citation-number initialization
   - year-suffix/disambiguation ordering
6. Leave citation-item ordering unchanged:
   - explicit `citation.sort` only
   - otherwise preserve citation input order
7. Add tests for:
   - author-date bibliography default uses title as tiebreak
   - note bibliography default uses title before year
   - citation order remains unchanged when `citation.sort` is absent
8. Update style author documentation to explain the split between
   `bibliography.sort` and `citation.sort`.

## Key Files

- `crates/citum-schema/src/options/processing.rs`
- `crates/citum-schema/src/options/mod.rs`
- `crates/citum-engine/src/processor/mod.rs`
- `docs/guides/style-author-guide.md`

## Notes

- Existing explicit style-level sorts remain valid and continue to win.
- Numeric insertion order remains stable because the engine bibliography uses
  `IndexMap`.
- This deliberately leaves room for a later citation-side ergonomic feature
  analogous to biblatex `sortcites`, but that should be a citation option, not
  a processing-family default.
