# Article-Journal No-Page Fallback Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-03-16
**Supersedes:** None
**Related:** `csl26-aa23`, `csl26-fk0w`

## Purpose

Specify a small external bibliography parameter for styles whose
`article-journal` bibliography output changes when page data is absent. The
goal is to preserve recurring legacy CSL behavior like Royal Society of
Chemistry's `if page ... else DOI` branch without introducing procedural
condition syntax into the production template surface. The feature is
intentionally narrow: it captures one repeated policy decision rather than
opening a general conditional language.

## Scope

In scope:

- one new bibliography option outside templates
- bibliography behavior for `article-journal` entries only
- runtime gating of existing semantic component kinds (`issued`, `volume`,
  `issue`, `pages`, `doi`) based on page presence
- migration of legacy CSL `if page ... else DOI` patterns into that option
- the Royal Society of Chemistry forcing case behind `csl26-aa23`

Out of scope:

- general conditional logic in templates
- arbitrary boolean expressions
- citation behavior
- non-`article-journal` fallback policies
- style-name-specific engine logic

## Design

### New option

Add a new structured bibliography option under `options.bibliography`.

Draft shape:

```yaml
options:
  bibliography:
    article-journal:
      no-page-fallback: doi
```

Proposed schema shape:

- `BibliographyConfig.article_journal: Option<ArticleJournalBibliographyConfig>`
- `ArticleJournalBibliographyConfig.no_page_fallback: Option<ArticleJournalNoPageFallback>`
- `ArticleJournalNoPageFallback::{None, Doi}`

The exact Rust/YAML field names are still reviewable, but the design intent is
fixed: this behavior belongs in bibliography options, not in template
components.

The first implementation may support only:

- `no-page-fallback: doi`

That is acceptable as long as the enum remains explicitly extensible for future
values such as `doi-or-url`, without allowing arbitrary expressions.

### Semantics

When `no-page-fallback: doi` is active:

1. It applies only to bibliography rendering.
2. It applies only to `article-journal` references.
3. It is evaluated after type-template selection, but before final component
   rendering.
4. If `page` is present, the current article detail path remains active.
5. If `page` is absent and DOI is present, the processor activates the DOI
   fallback path.
6. The behavior is replacement, not addition: the standard article detail block
   is swapped out for the DOI path.
7. If both `page` and DOI are absent, current behavior is preserved.

### Page absence

For this option, `page` is absent when the resolved page value for the reference
is missing or empty after normal value resolution. The option should not infer
page presence from volume, issue, or other adjacent fields.

### Active component policy

The option does not add a new template branch. Instead, it gates existing
semantic component kinds already visible to the processor.

For `article-journal` entries with `no-page-fallback: doi`:

- `date: issued`
- `number: volume`
- `number: issue`
- `number: pages`

belong to the standard article-detail set.

When `page` is absent and DOI is present, that detail set is suppressed for the
selected `article-journal` template, and the existing DOI component is allowed
to render in its place.

This keeps the decision outside templates while still reusing the current
template order, punctuation, and affixes.

### Template relationship

Templates still own:

- author, title, and container rendering
- component order
- punctuation and affixes
- explicit DOI formatting

The new option only decides which pre-existing semantic component set is active
for `article-journal` bibliography entries.

Engine responsibility:

- choose between the standard article detail block and the DOI fallback block
  when the option is enabled
- reuse the existing DOI value/rendering path

Template responsibility:

- define how the DOI component looks
- define how the normal year/volume/issue/pages cluster looks
- remain free of `page?` / `doi?` branching logic

### Migration mapping

`citum-migrate` should detect legacy CSL bibliography branches shaped like:

- `if variable="page"` → render year/volume/page detail block
- `else` + DOI rendering → render DOI fallback

For those styles, migration should:

1. emit the new bibliography option
2. preserve both the standard article-detail components and DOI component in
   the resulting article-journal type template
3. avoid flattening the branch into unconditional output

Migration must not emit the option for:

- unrelated journal styles
- additive patterns where DOI is always rendered and pages are merely appended
  when present
- patterns where the fallback is not DOI

### Initial forcing case

The first required consumer is `styles/royal-society-of-chemistry.yaml`.

For `ITEM-1` in `tests/fixtures/references-expanded.json`:

- legacy CSL expects DOI because `page` is absent
- current migrated style drops DOI and emits the wrong journal detail path
- the new option should restore the legacy path without adding template-level
  condition syntax

The legacy corpus contains additional styles using similar `else DOI` patterns,
so the feature should be implemented as a shared policy, not an RSC-only fix.

## Implementation Notes

- Put the public surface in `options::BibliographyConfig`, not in
  `template::Rendering` or template component structs.
- Keep the engine logic narrow and semantic: it should work from reference type
  plus component kind, not from style name.
- Reuse existing DOI value resolution and rendering instead of inventing a new
  output path.
- Migration should set the option only when the legacy branch is actually
  present.
- Treat this as a bounded feature, analogous in spirit to existing external
  bibliography/rendering parameters like `suppress-period-after-url` and
  `volume-pages-delimiter`.
- Resist turning this into a general policy DSL. If future work needs additional
  named fallbacks, add enum values or adjacent typed options instead.

## Acceptance Criteria

- [x] The style schema supports an external bibliography option for
      `article-journal` no-page fallback behavior.
- [x] `citum_engine` can suppress the standard article-detail component set and
      activate DOI output when the option is enabled and `page` is absent.
- [x] `citum-migrate` can map the legacy RSC `if page ... else DOI` branch to
      that option.
- [x] `royal-society-of-chemistry` reaches full bibliography fidelity for the
      `csl26-aa23` forcing case.
- [x] Styles without the option keep current behavior.

## Changelog

- v1.0 (2026-03-16): Initial draft based on review feedback to keep the feature
  outside templates.
- v1.0 (2026-03-16): Activated with schema, engine, migration, style, and
  behavior-test coverage for the RSC forcing case.
