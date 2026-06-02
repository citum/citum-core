# Disambiguation Specification

**Status:** Active
**Date:** 2026-05-29
**Related:** [`docs/reference/DISAMBIGUATION.md`](../reference/DISAMBIGUATION.md),
[`docs/specs/MULTILINGUAL.md`](./MULTILINGUAL.md),
[`docs/specs/MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`](./MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md),
[CSL styles#7667](https://github.com/citation-style-language/styles/issues/7667),
[CSL schema#452](https://github.com/citation-style-language/schema/issues/452)

## Purpose

Defines the normative model for disambiguation in Citum: when it activates, which
strategies are applied in which order, how keys are constructed, and how multilingual
and grouped bibliographies interact with it. The how-to reference for style authors
is [`docs/reference/DISAMBIGUATION.md`](../reference/DISAMBIGUATION.md); this
document is the design authority.

## Scope

**In scope:**
- Collision-key construction (what variables constitute a "same cite")
- Strategy cascade order and early-exit semantics
- Year-suffix assignment, including the issued-year-only keying rule
- Multilingual-aware key generation
- Group-aware suffix assignment and the `disambiguate: locally` option

**Out of scope:**
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

#### Year-suffix when the original-publication date differs

Our working assumption is that in author–date styles, year-suffix disambiguation
(a, b, c…) is based only on the author name(s) and the publication year of the
edition cited, not on any original-date or dual-date information. This matches the
general "same author, same year" rules and reprint guidance in major author–date
systems such as APA and Chicago, which treat the year of the edition consulted as
the operative year for citations and disambiguation. We are not aware of any major
style that clearly requires a different rule; if such a case emerges, we can revisit
this assumption and introduce a style-specific override.

For three reprints by one author — originally 1926, 1926, 1927, all published 1967
— Citum produces `(1926/1967a) (1926/1967b) (1927/1967c)`. Because only the `issued`
year enters `build_group_key`, all three reprints form one collision group and all
receive a suffix. The original-publication date is rendered as part of the output
but plays no role in the collision test.

citeproc-js produces `(1926/1967a) (1926/1967b) (1927/1967)` because it gates
suffix assignment on the full rendered date string; this diverges from our working
assumption and has no evidence of user dependence. See `div-009` in the
[Divergence Register](../adjudication/DIVERGENCE_REGISTER.md).

**Verification:** `apa_reprint_year_suffix_attaches_to_issued_year_only` in
`crates/citum-engine/tests/citations.rs` locks this behavior against regression.

### 2. Strategy cascade

Strategies are attempted in increasing order of disruptiveness and stop at the
first that resolves every collision in the group:

1. **Et-al expansion** (`names: true`) — reveal additional names beyond the
   et-al threshold.
2. **Given-name expansion** (`add_givenname: true`) — add initials or full given
   names when family-name collisions remain. Scoping controlled by
   `givenname-disambiguation-rule` (see §2.1).
3. **Year suffix** (`year_suffix: true`) — append a letter sequence (a–z, aa–az,
   …) to the issued year.

Each strategy is tried against the current collision group; if it splits the
group into singletons, no further strategies run. If et-al expansion produces
sub-groups that are still ambiguous, given-name expansion and/or year suffix are
applied to those sub-groups independently.

Implemented in `apply_group_hints` →
`try_apply_name_partitions` / `try_apply_givenname_resolution` /
`apply_year_suffix`.

### 2.1 `givenname-disambiguation-rule`

Specifies which author positions receive given-name expansion. The field lives on
`Disambiguation` in `citum-schema-style/src/options/processing.rs` as
`givenname_rule: GivennameRule`. Default: `by-cite`.

| CSL value | Engine scope | Notes |
|---|---|---|
| `by-cite` *(default)* | expand all positions | **diverges from spec** — see §2.1.1 |
| `all-names` | expand all positions | same as current `by-cite` engine behavior |
| `all-names-with-initials` | expand all positions | initials vs full controlled by contributor config `initialize-with` |
| `primary-name` | expand **first author only** | required by Chicago author-date |
| `primary-name-with-initials` | expand **first author only** | required by APA 7; initials via contributor config |

**Key invariant:** initials vs full given name is always driven by the contributor
config's `initialize-with` / `name-form` settings, not by this rule. The rule
controls only *which positions* are eligible for expansion.

#### 2.1.1 `by-cite` divergence (csl26-lvib)

**CSL spec** (1.0.1+): `by-cite` is per-cite and minimal. For each rendered
citation, the engine expands only the minimum given-name subset needed to
disambiguate *that specific cite* from the others currently in scope. Two
cites of colliding works may expand different subsets of their name lists —
only what is strictly necessary for each.

**Current engine**: `apply_group_hints` processes the full bibliography as a
single collision group and sets `expand_given_names: true` on every reference
in the group. This is effectively `all-names` behavior — any work involved in
*any* same-author-same-year collision gets given-name expansion globally,
regardless of whether the specific citation in context actually needs it.

**Practical impact**: Low for typical documents. The divergence is visible
when a colliding author also appears in non-colliding cites: CSL `by-cite`
would leave the non-colliding cite unexpanded; the current engine expands it.
No major style's oracle currently catches this gap because all tested styles
use `by-cite` as a default that has no observable effect when only one form
of the name appears.

**Implementation path** (bean csl26-lvib): the disambiguator would need to
shift from bibliography-wide collision groups to a per-citation rendering pass
that lazily computes the minimal expansion set for each cite in context.
This is a non-trivial engine refactor — deferred until a concrete style
failure demonstrates the need.

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

### 6. Short-title suppression via `first-reference-note-number`

A note style may add a short title *only* to resolve a collision, but that same
title should not appear in `first-reference-note-number` cross-ref citations,
where the note number is already sufficient identification.

**Example** (two works by same author, same year):

| Context | Output |
|---|---|
| First cite of *Rome* | Smith, *Rome*, 2020, 45. |
| First cite of *Greece* | Smith, *Greece*, 2020, 67. |
| Later short cite of *Rome* | Smith, see n. 1. |

**Implementation:** `ProcHints.suppress_disambiguation_title` is set when a
subsequent-position citation has a `first_reference_note_number` (populated by
`normalize_note_context` from the `first_note_by_id` map on `Processor`).
The renderer in `values/title.rs` checks this flag and suppresses any template
title component with `disambiguate_only: true`. The first-reference note number
itself is available as `number: first-reference-note-number` in templates.
Suppression is gated on the template actually rendering the note-number identifier
(`template_uses_first_ref_note_number`) to prevent silent reintroduction of
ambiguity.

This suppression is automatic and not currently style-configurable; if a future
style needs different behavior, a `suppress_disambiguation_title` option can be
added to the citation context.

## Implementation Notes

- The collision-key layer is intentionally separated from the rendering layer so
  that disambiguation decisions are reproducible without re-rendering all
  references.
- `DisambiguationFlags` (derived from `Disambiguation` struct) is the only
  per-style knob passed into the disambiguator; it must not grow to include
  rendering concerns.
- All native disambiguation tests in `crates/citum-engine/tests/citations.rs`
  (the `citations` nextest target) pass.

## Acceptance Criteria

- [x] Year-suffix collision key uses only `issued` year (no original-date gate)
- [x] All native disambiguation tests passing
- [x] Group-aware suffix restart implemented (`disambiguate: locally`)
- [x] Multilingual key generation respects display mode
- [x] Native fixture asserting `(1926/1967a) (1926/1967b) (1927/1967c)` for the APA §8.15 reprint scenario
- [x] Short-title suppression via `first-reference-note-number` implemented and tested
- [x] `givenname-disambiguation-rule` field exists on `Disambiguation`; `primary-name` and
  `primary-name-with-initials` restrict expansion to the first author only (csl26-4ada)

## Changelog

- 2026-06-02: Added §2.1 `givenname-disambiguation-rule` (csl26-4ada). Documents
  `GivennameRule` enum (5 CSL values), engine's two-scope collapse
  (primary vs all), and `by-cite` per-cite minimal-subset as a documented
  divergence. Added acceptance criterion for primary-name scoping.
- 2026-05-31: Implemented `render_name_for_disambiguation` (csl26-54jn). Flattens
  contributors via `resolve_multilingual_name` so the collision key matches the style's
  active display mode (transliterated/translated/primary). Covered by
  `test_multilingual_key_generation_respects_display_mode` in
  `crates/citum-engine/src/processor/disambiguation.rs`. All acceptance criteria now met.
- 2026-05-31: Test soundness audit (csl26-ucs3). Corrected `[x]` → `[ ]` for
  multilingual key generation — `render_name_for_disambiguation` not yet
  implemented; `disambiguation.rs` always reads `Contributor::Multilingual.original`.
  See `docs/architecture/audits/2026-05-31_DISAMBIGUATION_TEST_SOUNDNESS.md`.
- 2026-05-29: Initial version. Consolidates `DISAMBIGUATION_IMPLEMENTATION_PLAN.md` (now deleted)
  and `DISAMBIGUATION_MULTILINGUAL_GROUPING.md` (now deleted).
- 2026-05-29: All acceptance criteria implemented; status set to Active.
- 2026-05-29: Removed `disambiguate.ignore` (doubly-redundant no-op option); added `div-009`
  to Divergence Register grounding the issued-only keying decision in APA §8.15 / Chicago.
