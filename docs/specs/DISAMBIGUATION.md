# Disambiguation Specification

**Status:** Draft
**Date:** 2026-05-29
**Related:** [`docs/reference/DISAMBIGUATION.md`](../reference/DISAMBIGUATION.md),
[`docs/specs/MULTILINGUAL.md`](./MULTILINGUAL.md),
[`docs/specs/MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`](./MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md),
[CSL styles#7667](https://github.com/citation-style-language/styles/issues/7667),
[CSL schema#452](https://github.com/citation-style-language/schema/issues/452)

## Purpose

Defines the normative model for disambiguation in Citum: when it activates, which
strategies are applied in which order, how keys are constructed, how multilingual
and grouped bibliographies interact with it, and how the two corner cases flagged
by CSL schema maintainers are addressed. The how-to reference for style authors
is [`docs/reference/DISAMBIGUATION.md`](../reference/DISAMBIGUATION.md); this
document is the design authority.

## Scope

**In scope:**
- Collision-key construction (what variables constitute a "same cite")
- Strategy cascade order and early-exit semantics
- Year-suffix assignment, including the issued-year-only keying rule (Case A)
- Multilingual-aware key generation
- Group-aware suffix assignment and the `disambiguate: locally` option
- The `disambiguate.ignore` option design for generalized signature exclusion (Case B — design locked, implementation deferred)

**Out of scope:**
- Short-title-for-disambiguation combined with `first-reference-note-number` cross-refs (tracked separately as a follow-up bean)
- Rendering specifics (see [`docs/reference/DISAMBIGUATION.md`](../reference/DISAMBIGUATION.md))

## Design

### 1. Collision key

A collision group is a set of references that share the same **author key** and
**year key**. These keys are computed in
[`processor/disambiguation.rs`](../../crates/citum-engine/src/processor/disambiguation.rs)
and are the only inputs to the disambiguation decision — the *rendered output* is
never consulted.

**Author key** (`build_author_key`): lowercased family names of contributing
names, joined by commas, with the et-al suffix included when the name list is
abbreviated by the style's `shorten` config. This matches what the style will
visually show, so disambiguation keys track rendered output without re-rendering.

**Year key** (`build_group_key`): the `issued` year (`csl_issued_date()`),
appended to the author key. **No other date field participates in the key.**

#### Case A — year-suffix when original-publication date differs (APA §8.15)

citeproc-js applies year-suffix only when the *full rendered* citation string
(including any original-date component such as `1926/1967`) is identical. This
means three reprints by one author — originally 1926, 1926, 1927, all published
1967 — receive suffixes on only the first two, producing `(1926/1967a)
(1926/1967b) (1927/1967)`.

APA §8.15 requires `(1926/1967a) (1926/1967b) (1927/1967c)`: the letter follows
the *published* year, regardless of the original date.

**Citum's keying is already APA-correct by design.** Because only `issued` year
enters `build_group_key`, the three reprints form one collision group and all
three receive a suffix. The original-publication date is rendered as part of the
output but plays no role in the collision test.

No option is provided for citeproc-js parity. The legacy behavior conflates the
collision key with the rendered string; that conflation is the defect, not a
feature worth porting.

**Verification:** a native integration test in
`crates/citum-engine/tests/citations.rs` (filed as a follow-up bean) locks this
behavior against regression.

### 2. Strategy cascade

Strategies are attempted in increasing order of disruptiveness and stop at the
first that resolves every collision in the group:

1. **Et-al expansion** (`names: true`) — reveal additional names beyond the
   et-al threshold.
2. **Given-name expansion** (`add_givenname: true`) — add initials or full given
   names when family-name collisions remain. Controlled by
   `givenname-disambiguation-rule: by-cite | all-names`.
3. **Year suffix** (`year_suffix: true`) — append a letter sequence (a–z, aa–az,
   …) to the issued year.

Each strategy is tried against the current collision group; if it splits the
group into singletons, no further strategies run. If et-al expansion produces
sub-groups that are still ambiguous, given-name expansion and/or year suffix are
applied to those sub-groups independently.

Implemented in `apply_group_hints` →
`try_apply_name_partitions` / `try_apply_givenname_resolution` /
`apply_year_suffix`.

### 3. Year-suffix assignment ordering

Within a collision group, suffixes are assigned by a deterministic sort. The
default sort key is `title` (lowercased). When a `BibliographyGroup` defines a
`sort`, that sort takes precedence within the group (see §5 below).

The letter sequence is base-26 with wrapping: 1→a, 26→z, 27→aa, 52→az, 53→ba,
… Implemented in `int_to_letter` in `values/date.rs`.

### 4. Multilingual-aware keys

When the style's multilingual config specifies a display mode other than
`primary`, the author key must reflect the same surface form the style will
render. `render_name_for_disambiguation` selects the appropriate name variant
(transliteration, translation, or original) before lowercasing and joining.

This ensures that if a style shows transliterated names, two references whose
transliterations collide are treated as a disambiguation collision, not two
distinct authors.

Monolingual references (no `MultilingualComplex`) always fall through to the
original via the `Display` trait chain.

### 5. Group-aware disambiguation (`disambiguate: locally`)

When a `BibliographyGroup` sets `disambiguate: locally`, the disambiguator is
instantiated per group rather than globally. Consequences:

- **Year-suffix sequences restart** at `a` within each group. A "Cases" group
  and a "Books" group can each begin `(2020a) (2020b)` independently.
- **Group sort** (if set) drives suffix ordering instead of the global title
  sort.
- No suffix escapes a group's boundary; within-group collision detection is
  scoped to that group's reference set.

Without `disambiguate: locally`, disambiguation runs globally across the full
bibliography.

### 6. `disambiguate.ignore` — generalized signature exclusion (design, not yet implemented)

CSL schema maintainers proposed a per-element template attribute
`ignore-for-disambiguation="true"` ([schema#452](https://github.com/citation-style-language/schema/issues/452)). Citum's approach is an
**option** that names which reference variables are excluded from the collision
signature, keeping the template language declarative.

Proposed option (in `Disambiguation` struct,
`crates/citum-schema-style/src/options/processing.rs`):

```yaml
citation:
  options:
    disambiguate:
      year-suffix: true
      ignore: [original-date]   # variables excluded from collision key
```

Semantics: any variable listed under `ignore` is not consulted when building
`build_group_key`. Practically, the initial implementation targets `original-date`
as the only meaningful value (it is the only variable that current citeproc-js
conflates into its collision gate).

The `ignore` list is checked inside `build_group_key` before year key
construction; if `original-date` is listed, `csl_issued_date()` is still used
(Citum's default) and the option becomes a no-op — the option's primary value is
as an explicit, self-documenting override for migrated CSL styles that shipped
with a workaround.

**Implementation path (deferred):**
1. Add `ignore: Option<Vec<ReferenceVariable>>` to `Disambiguation` in
   `crates/citum-schema-style/src/options/processing.rs:393`.
2. Thread into `DisambiguationFlags` and plumb to `build_group_key`.
3. Regenerate JSON schemas (`cargo run --bin citum --features schema -- schema
   --out-dir docs/schemas`).

#### Sub-case: short-title + `first-reference-note-number` (open question)

A note style may add a short title *only* to resolve a collision, but that same
title should not appear in `first-reference-note-number` cross-ref citations,
where the note number is already sufficient identification.

**Example** (two works by same author, same year):

| Context | Output |
|---|---|
| First cite of *Rome* | Smith, *Rome*, 2020, 45. |
| First cite of *Greece* | Smith, *Greece*, 2020, 67. |
| Later short cite of *Rome* — desired | Smith, see n. 1. |
| Later short cite of *Rome* — current gap | Smith, *Rome*, see n. 1. ← title redundant |

Neither CSL nor Citum currently have a way to express "show short title when
disambiguating AND suppress it in cross-ref position." The note number and the
disambiguation title are produced by independent layers with no joint policy.

**Implementation sketch:** a new `show_short_title` variant on `ProcHints` (set
by the disambiguator when name/year strategies fail). The renderer emits the
short title when the hint is set. The cross-ref rendering path
(`processor/document/note_support.rs`) would need to check position context and
suppress `show_short_title` hints when a `first-reference-note-number` is
present — the note number supersedes the title as the identifier. This requires
`ProcHints` to carry a position-aware suppression flag, or the note-number
assignment pass to clear `show_short_title` on affected citations after the fact.

Tracked as `csl26-xrc5`; **not** addressed by the `ignore` option above.

## Implementation Notes

- The collision-key layer is intentionally separated from the rendering layer so
  that disambiguation decisions are reproducible without re-rendering all
  references.
- `DisambiguationFlags` (derived from `Disambiguation` struct) is the only
  per-style knob passed into the disambiguator; it must not grow to include
  rendering concerns.
- 11/11 native disambiguation tests in `crates/citum-engine/tests/citations.rs`
  (the `citations` nextest target) pass as of 2026-05-29.

## Acceptance Criteria

- [x] Year-suffix collision key uses only `issued` year (no original-date gate)
- [x] 11/11 native disambiguation tests passing
- [x] Group-aware suffix restart implemented (`disambiguate: locally`)
- [x] Multilingual key generation respects display mode
- [ ] Native fixture asserting `(1926/1967a) (1926/1967b) (1927/1967c)` for the APA §8.15 reprint scenario
- [ ] `disambiguate.ignore` option implemented and schema regenerated
- [ ] `disambiguate.ignore` covered by tests

## Changelog

- 2026-05-29: Initial version. Consolidates `DISAMBIGUATION_IMPLEMENTATION_PLAN.md` (now deleted)
  and `DISAMBIGUATION_MULTILINGUAL_GROUPING.md` (now deleted); adds Case A and Case B design.
