# Type-Classification Centralization Specification

**Status:** Active
**Version:** 1.1
**Date:** 2026-07-05
**Supersedes:** None
**Related:** `csl26-92mg`, `csl26-8m2p`, audit `docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md` (Finding 14)

## Purpose

Replace six inconsistent, engine-hardcoded per-reference-type presentation
rules with a single authoritative classification surface, and replace the
English-string `[Dataset]` suffix hack with a localizable, term-driven
type-label. This is an internal-design correction, not an effort to match
citeproc output — the goal is that adding or renaming a reference type
changes rendering in exactly one place, and that no user-visible label text
is an English literal embedded in engine code or in a style's `suffix`.

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
module without changing behavior. Site 1 is a small localizable feature.
Citum currently collapses the dataset type-label into an English literal
glued to the title, which then requires an engine-side de-dup against the
synthesized `[Untitled dataset]` value produced for titleless datasets
(`citum-schema-data/src/reference/conversion/scholarly.rs:846-857`). A
generic "suppress this suffix" style flag was considered and rejected: it
still encodes the label text as an English literal in the style, which is
the coupling this spec exists to remove.

As prior art (not a normative target): citeproc/APA's `[Dataset]` is a
bracketed **description** group emitting a localized `term="dataset"` with a
`genre`/`medium` fallback (see `styles-legacy/apa.csl`, macros `description`
and `description-format-term-generic`). Citum's fix independently arrives at
the same shape — a localized, term-driven label — because it is the correct
internal design, not because it reproduces citeproc.

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
  here, dropping the `contains("ancient")` fuzzy match for an explicit member
  list (`classic`, `religious-text`). `ref_type()`
  (`citum-schema-data/src/reference/accessors.rs:1432`) emits a finite,
  enumerable CSL-string set derived from `ReferenceClass` + genre; the
  `Classic` class always emits the literal string `"classic"`, so
  `ref_type()` never produces a string containing `"ancient"` — the fuzzy
  match matches nothing the data model can produce and is dead weight.
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
`render/component.rs:271-283`. Add a new **`TypeLabel`** template component
whose text is a *localized term* for the reference's own type, with a
`genre`/`medium` fallback, rendered like any other component (`rendering`
carries `prefix`/`suffix`/`wrap`/`text-case` as normal).

The component itself emits only the resolved term text. Brackets are **not**
baked into the component — they come from the existing `wrap: brackets`
rendering option (`Rendering.wrap: Option<WrapConfig>`,
`citum-schema-style/src/template.rs:127-129`), exactly like any other
bracketed component in a style. This keeps the component reusable for styles
that don't bracket type labels.

Citum already has: `TemplateComponent::Term`, per-type template-variants,
locale vocab for `genre`/`medium` (`locale/vocab.rs`), and term lookup. A
dedicated `TypeLabel` variant (rather than overloading `Term`, which
resolves a *named* term, not "the term for this reference's own type") keeps
the "fixed term" and "type-driven term" cases distinct and each simple.

The dataset type-variant in `apa-7th.yaml` becomes, conceptually:

```yaml
dataset:
- contributor: author   # unchanged
- date: issued          # unchanged
- title: primary        # no more " [Dataset]." suffix
- type-label:            # new: resolves the localized term for this
                          # reference's type (genre/medium fallback first)
    wrap: brackets
- variable: publisher   # unchanged
- variable: url         # unchanged
```

Because the label is now its own component sourced from the locale, the
titleless-dataset duplication disappears at its root: the `[{genre}]` value
synthesized in `scholarly.rs:846-857` and the type-label are distinct,
locale-driven elements, and the fallback rule (prefer genre/medium, else the
type term) prevents `[Untitled dataset] [Dataset]`.

The `dataset` type term already exists in the shipped locale
(`crates/citum-schema-style/embedded/locales/en-US.yaml:406`), so Part B's
implementation has a term to resolve against without also authoring new
locale content.

## Design Decisions (resolved 2026-07-05, PR #1008 review)

1. **Sequencing.** Two PRs: Part A (schema-neutral centralization) lands
   first; Part B (`feat`, `TypeLabel` component) follows as a separate PR so
   the mechanical centralization is not gated on the label design.
2. **Type-label component shape.** A new dedicated `TemplateComponent::TypeLabel`
   that resolves the reference-type term with genre/medium fallback, using
   standard `Rendering` (`wrap`, `text-case`, etc.) like every other
   component — brackets are a style-level `wrap: brackets`, not baked into
   the component.
3. **`type_class` module home.** `values/type_class.rs`, next to today's
   `type_class_matches`.
4. **`contains("ancient")` fate.** Replaced with an explicit member list
   (`classic`, `religious-text`). `ref_type()` is a finite, enumerable
   CSL-string set (see Part A above) and never produces a string containing
   `"ancient"`, so the fuzzy match is confirmed dead weight, not a
   forward-compat hedge.

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
- v1.1 (2026-07-05): Reframed away from citeproc-matching as a goal (prior
  art only); clarified `TypeLabel` emits term text only, brackets via
  `wrap: brackets`; resolved all four open decisions into settled Design
  Decisions following PR #1008 review; confirmed `ref_type()` vocabulary is
  finite and never emits `"ancient"`. Status: Draft → Active.
