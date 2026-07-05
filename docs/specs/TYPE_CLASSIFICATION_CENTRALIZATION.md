# Type-Classification Centralization Specification

**Status:** Draft
**Version:** 1.0
**Date:** 2026-07-05
**Supersedes:** None
**Related:** `csl26-92mg`, `csl26-8m2p`, audit `docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md` (Finding 14)

## Purpose

Replace six inconsistent, engine-hardcoded per-reference-type presentation
rules with a single authoritative classification surface, and replace the
English-string `[Dataset]` suffix hack with a localizable, term-driven
type-label. The goal is that adding or renaming a reference type changes
rendering in exactly one place, and that no user-visible label text is an
English literal embedded in engine code or in a style's `suffix`.

## Scope

In scope:
- The six sites enumerated in the audit (see table below).
- One engine classification module owning ref_type → { title-category,
  TypeClass membership, serial-parent-ness, selector aliases, DOI-URL
  synthesis } as data, not scattered `match`/`contains` arms.
- A localizable type-label mechanism to retire `apa-7th.yaml`'s
  `suffix: " [Dataset]."` and the engine de-dup that pairs with it.

Out of scope:
- A general conditional/expression language in templates.
- Reworking the CSL→Citum type *conversion* contract
  (`docs/specs/CSL_TYPE_CONVERSION_CONTRACT.md`) beyond what site 1 needs.
- Non-dataset description labels (interview, thesis, review, …) — the
  mechanism is designed to generalize to them, but only `dataset` is
  migrated in the first implementation.
- Any change to citation (as opposed to bibliography) output.

## The six sites (verified against HEAD, 2026-07-05)

| # | Site | Current hardcoding |
|---|------|--------------------|
| 1 | `render/component.rs:271-283` `get_effective_rendering` | `[Dataset]` literal suffix rewrite (ref_type + value-`[` + Primary title + literal string) |
| 2 | `values/title.rs:126` `parent_short_title` | `ref_type().contains("article") \|\| == "broadcast"` gate on `ParentSerial` |
| 3 | `values/locator.rs:235-256` `type_class_matches` | hardcoded legal/classical ref_type lists incl. `ref_type.contains("ancient")` |
| 4 | `render/component.rs:293-404` `get_title_category_rendering` | "Legacy hardcoded logic" ref_type→category fallback tables (ContainerTitle / ParentSerial / Primary) |
| 5 | `values/variable.rs:222-226` `SimpleVariable::Url` | DOI-URL synthesis gated on `ref_type() == "dataset"` |
| 6 | `processor/rendering/grouped/component_predicates.rs:36-41` `aliased_type_selector_candidates` | `chapter` silently aliases to `entry-dictionary` type-variants |

Sites 2–6 are mechanical centralization: moving existing logic into one
module without changing behavior. Site 1 is a small localizable feature —
citeproc/APA's `[Dataset]` is not a title suffix but a bracketed
**description** group emitting a localized `term="dataset"` with a
`genre`/`medium` fallback (see `styles-legacy/apa.csl`, macros `description`
and `description-format-term-generic`). Citum currently collapses that into
an English literal glued to the title, which then requires an engine-side
de-dup against the synthesized `[Untitled dataset]` value produced for
titleless datasets (`citum-schema-data/src/reference/conversion/scholarly.rs:846-857`).
A generic "suppress this suffix" style flag was considered and rejected: it
still encodes the label text as an English literal in the style, which is the
coupling this spec exists to remove.

## Design

### Part A — Single classification module (sites 2–6)

Introduce `crate::values::type_class` (engine-internal, `pub(crate)`), the
single home for reference-type classification facts. Proposed surface:

- `title_category(ref_type: &str) -> TitleCategory` — the default
  ref_type→category mapping currently inlined as the "Legacy hardcoded logic"
  fallback in `get_title_category_rendering`. `get_title_category_rendering`
  keeps consulting the style's declarative `titles.type_mapping` first, then
  falls back to this table (behavior-preserving).
- `matches_type_class(ref_type, TypeClass) -> bool` — moves `type_class_matches`
  here verbatim (dropping the `contains("ancient")` fuzzy match in favor of an
  explicit member list; see Open Decisions).
- `is_serial_parent_type(ref_type) -> bool` — replaces
  `ref_type().contains("article") || == "broadcast"` in `parent_short_title`
  with an explicit list aligned to the `periodical`/`serial` category.
- `type_selector_aliases(ref_type) -> &'static [&'static str]` — moves the
  `chapter → [chapter, entry-dictionary]` alias table out of
  `component_predicates.rs`.
- `synthesizes_doi_url(ref_type) -> bool` — replaces the `== "dataset"` gate in
  `SimpleVariable::Url`.

Each function is table-driven and documented, so the six sites become thin
call-throughs. `TitleCategory` is an engine enum mirroring the existing
category strings (`component`, `monograph`, `periodical`, `serial`,
`collection`, `default`) so it can co-exist with the string-keyed
`titles.type_mapping` without a schema change.

This part is schema-version-neutral and touches no style YAML or locale data.

### Part B — Localizable type-label (site 1)

Retire `apa-7th.yaml`'s `suffix: " [Dataset]."` and the engine de-dup at
`render/component.rs:271-283`. Model the label the way citeproc does: a
bracketed **type-label** element rendered after the title, whose text is a
*localized term*, with a `genre`/`medium` → reference-type-term fallback.

Citum already has: `TemplateComponent::Term`, per-type template-variants,
locale vocab for `genre`/`medium` (`locale/vocab.rs`), and term lookup. The
missing capability is a component that resolves *the term for the reference's
own type* (with the genre/medium fallback) rather than a fixed term name.

The dataset type-variant in `apa-7th.yaml` becomes, conceptually:

```yaml
dataset:
- contributor: author   # unchanged
- date: issued          # unchanged
- title: primary        # no more " [Dataset]." suffix
- <type-label>          # new: localized term, wrapped in brackets
- variable: publisher   # unchanged
- variable: url         # unchanged
```

Because the label is now its own component sourced from the locale, the
titleless-dataset duplication disappears at its root: the `[{genre}]` value
synthesized in `scholarly.rs:846-857` and the type-label are distinct,
locale-driven elements, and the fallback rule (prefer genre/medium, else the
type term) prevents `[Untitled dataset] [Dataset]`.

The exact component shape is the main open decision (see below).

## Implementation Notes

- Part A is behavior-preserving: existing snapshot/oracle output must not
  move. `test_scholarly_fixture_*` in `tests/domain_fixtures.rs` and the APA
  dataset tests in `tests/bibliography.rs` are the guard rails.
- Part B changes user-visible output paths for datasets and bumps
  `STYLE_SCHEMA_VERSION` (new template component / option). Do **not** bump it
  by hand — it is inferred from conventional commits; the `feat` commit that
  introduces the component carries the minor bump. Regenerate schemas
  (`just schema-gen`) in the same commit.
- `apa-7th.yaml` exists both at `styles/apa-7th.yaml` and
  `crates/citum-schema-style/embedded/styles/apa-7th.yaml`; both must change
  together and stay in sync.
- Oracle: APA is CSL-derived → CSL oracle (`node scripts/oracle.js`). Verify
  the dataset label against citeproc-js for both titled and titleless
  datasets.
- The synthesized `(Version X)` string in `scholarly.rs:854` is a second
  English literal in the same area; note it as adjacent debt but leave it to
  a follow-up unless the type-label work forces it.

## Open Decisions

1. **Sequencing.** (a) Two PRs — land Part A (schema-neutral) first, then
   Part B as a `feat`; or (b) one combined PR. Recommendation: **(a)**, so
   the mechanical centralization is not gated on the type-label design.
2. **Type-label component shape for Part B.** Options:
   - A new dedicated `TemplateComponent` (e.g. `TypeLabel`) that resolves
     the reference-type term with genre/medium fallback and standard
     wrap/text-case rendering. Cleanest semantic match to citeproc's
     `description` macro; largest schema surface.
   - Extend `TemplateComponent::Term` with a "term = this reference's type"
     selector plus a fallback list, reusing existing term rendering. Smaller
     surface; overloads `Term`.
   - Keep the current title-suffix mechanism but source the label from a
     locale term instead of the literal string (interim, removes
     locale-coupling only; still a de-dup hack). Smallest; least correct.
3. **`type_class` module home.** `values/type_class.rs` (next to today's
   `type_class_matches`) vs. a crate-root `type_class.rs`. Recommendation:
   **`values/type_class.rs`**.
4. **`contains("ancient")` fate.** Replace with an explicit member list
   (e.g. `ancient`, `ancient-text`?) — needs the actual set of classical
   ref_types Citum recognizes. Requires confirming the type vocabulary
   before Part A lands.

## Acceptance Criteria

- [ ] All six audit sites call into the centralized module / new component;
      no per-type `match`/`contains` on `ref_type` remains at those sites.
- [ ] No user-visible label text is an English literal in engine code or in
      a style `suffix` for the dataset case.
- [ ] Part A produces byte-identical output to HEAD across the engine test
      suite and the APA oracle (titled + titleless datasets).
- [ ] Adding a new reference type requires editing exactly one table to
      affect title-category, TypeClass, serial-parent, selector-alias, and
      DOI-URL behavior.
- [ ] `just pre-commit` clean; schemas regenerated if Part B lands.

## Changelog
- v1.0 (2026-07-05): Initial draft.
