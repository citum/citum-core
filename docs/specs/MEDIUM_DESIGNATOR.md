# Medium Designator Specification

**Status:** Active
**Version:** 1.1
**Date:** 2026-09-08
**Supersedes:** None
**Related:** `csl26-zs9y`, `csl26-8z39`, `csl26-e8ul`,
`docs/architecture/audits/2026-09-06_RENDER_WHEN_DISPOSITION.md`,
`docs/specs/ARTICLE_JOURNAL_NO_PAGE_FALLBACK.md`

## Purpose

The NLM/Vancouver citation family marks any reference it has only ever
accessed online with two things, both keyed on "does this reference have
a URL":

- an `[Internet]` marker, bracketed onto the title or the container title;
- a `[cited …]`-style bracket around the access date.

This is a bundle, not a fallback: the marker appears *in addition to* the
title, never instead of it, so it's not covered by `docs/specs/GROUP_SELECT.md`.
It covers embedded exact-parity rows across three styles —
`taylor-and-francis-national-library-of-medicine`,
`springer-vancouver-brackets`, and
`taylor-and-francis-council-of-science-editors-author-date` — that differ
from each other in exactly two ways: which reference types get the
`[Internet]` marker, and which locale term backs the accessed-date bracket.
This spec proposes one bibliography option, `online-access`, that captures
the bundle and lets each style set its own answers to those two
differences.

**Not in this bundle:** how each style renders the URL/DOI itself (a
"Retrieved from:"/"Accessed DATE"/bare-URL phrase, depending on style) —
see Evidence and Scope below for why that's genuinely different work, not
a third shared field.

## Scope

In scope: one bibliography option controlling the marker and the
cited-date bracket for the vancouver/NLM style family; the rule for which
title the marker attaches to.

Out of scope:

- other families' access-date conventions (Chicago's URL/DOI policy is
  tracked separately under `csl26-h7oc`);
- a general-purpose conditional-literal mechanism — like
  `ARTICLE_JOURNAL_NO_PAGE_FALLBACK.md`, this proposes one narrow named
  option, not a reusable primitive;
- NLM's rule for using a DOI instead of the usual page/volume detail block
  — unrelated to URLs or this marker; tracked under `csl26-8z39`;
- how the URL or DOI itself renders — NLM's "Retrieved from: URL" (itself
  suppressed for `article-journal`), Springer's "URL. Accessed DATE"
  (DOI preferred over URL), and CSE's bare URL (DOI preferred) are three
  genuinely different rules, not wording variants of one shared phrase —
  see Evidence. Tracked as its own design problem in `csl26-e8ul`,
  entangled with `csl26-8z39`'s DOI-preference logic for NLM specifically.

## Evidence

NLM (`taylor-and-francis-national-library-of-medicine.csl:133-150`) marks
`[Internet]` on the title whenever a URL exists, for every reference type
*except* five periodical/monograph types (`article-journal`,
`article-magazine`, `chapter`, `paper-conference`, `article-newspaper`) —
those already carry enough locating detail (volume, issue, page) that the
marker would be redundant. Springer follows the same rule.

T&F-CSE (`taylor-and-francis-council-of-science-editors-author-date.csl:58-125`)
reaches the same outcome a different way: no type list at all. It marks
`container` when `container-title` is present, and `title` only when it
isn't:

```xml
<macro name="container">
  <text variable="container-title" form="short" strip-periods="true"/>
  <choose>
    <if variable="URL">
      <text term="internet" prefix=" [" suffix="]" text-case="capitalize-first"/>
    </if>
  </choose>
</macro>
```

These agree in practice — see "Anchor selection" below — because the five
excluded types are exactly the types that carry an embedded container
title. T&F-CSE's data-presence rule is the one this spec adopts, since it
generalizes; NLM/springer's type list becomes a consequence of it, not a
separate rule to encode.

T&F-CSE also names a *different* locale term for the accessed-date
bracket. Its macro is confusingly also called `cited`, but renders
`term.accessed`, not `term.cited` like NLM and springer do
(`taylor-and-francis-council-of-science-editors-author-date.csl:96-105`).
A shared option that hardcodes one term for all three styles gets one of
them wrong — see "Cited-date label" below.

**Why URL/DOI rendering isn't part of this bundle.** Each style's `access`
macro — the part that actually renders the URL or DOI — is a different
rule, not a wording variant of one shared phrase:

- NLM (`taylor-and-francis-national-library-of-medicine.csl:72-91`): for
  `article-journal`, render a DOI if page/volume are both absent,
  otherwise nothing (`csl26-8z39`'s scope); for every other type with a
  URL, "Retrieved from: URL". The phrase never renders for
  `article-journal`, regardless of URL.
- Springer (`springer-vancouver-brackets.csl:88-107`): DOI if present,
  rendered as a `doi.org` URL; otherwise, if a URL exists, "URL. Accessed
  DATE" — no "retrieved" wording, and a date folded into this same macro,
  separate from the `[cited …]` bracket rendered elsewhere in the entry.
- T&F-CSE (`taylor-and-francis-council-of-science-editors-author-date.csl:48-56`):
  DOI if present, as a `doi.org` URL; otherwise a bare URL. No phrase, no
  date.

A single `access-phrase: bool` toggle can't represent three rules this
different — it would either add wording NLM itself suppresses for
`article-journal`, or add "Retrieved from" phrasing Springer and CSE never
use at all. This needs its own design, tracked in `csl26-e8ul`.

## Design

```yaml
bibliography:
  options:
    online-access:
      medium-marker: {message: term.internet}
      cited-date-label: {message: term.cited}   # {message: term.accessed} for T&F-CSE
      cited-date-form: year-month-abbr-day   # CSL's form="text" shape: "2024 Jan 15"
```

- `medium-marker`: locale message rendered bracketed and capitalized-first,
  anchored per "Anchor selection" below.
- `cited-date-label`: locale message naming the term inside the
  accessed-date bracket (see "Cited-date label" below). Omitting it
  disables the bracket even when a URL exists.
- `cited-date-form`: date form for the accessed-date bracket; only
  meaningful when `cited-date-label` is set. Accepts any `TemplateDate`
  `form` value; `year-month-abbr-day` reproduces CSL's `form="text"` shape
  used by both NLM's and CSE's real macros (added during implementation —
  see Implementation Notes).

When a reference has no URL, neither renders — unaffected for any style
that doesn't set this option.

### Anchor selection

The marker attaches to the container title when
`reference.container_title()` returns one, and to the reference's own
title otherwise — the same data T&F-CSE's own rule tests. This is not a
type classification; it's the actual accessor the container-title
component itself already consults, so it can't drift out of sync with a
type list the way a hardcoded classification could.

**Known gap, gated by Acceptance Criteria below — not resolved in this
section:** legal reference types (bills, statutes, regulations, treaties)
also populate `container_title()`, but from a flat reporter/code string,
not an embedded work — and NLM's real behavior treats these as
title-anchored, not container-anchored. The anchor rule needs to
distinguish "container is an embedded titled work" from "container is a
flat citation string" before it's correct for these types. This option
cannot promote to Active until that's fixed and covered by the fixture
Acceptance Criteria requires — see Implementation Notes.

### Cited-date label

NLM and springer name `term.cited`; T&F-CSE names `term.accessed` — two
distinct, already-existing locale terms, not a wording variant of the
same one. Each style sets `cited-date-label` to name its own term; no
locale override is needed since both terms already exist in the base
locale.

### Semantics

When a reference has a URL: the medium marker renders bracketed and
capitalized-first on whichever title "Anchor selection" picks; if
`cited-date-label` is set, the accessed date renders bracketed using it
and `cited-date-form`.

## Implementation Notes

*(for engineers implementing this spec — not required reading to
understand the design)*

- New type `OnlineAccessConfig` (fields: `medium_marker`,
  `cited_date_label`, `cited_date_form`), added to **three** places,
  following `ArticleJournalBibliographyConfig`'s exact precedent:
  `BibliographyOptions` (`options/mod.rs:365`, the authoring-time type),
  `BibliographyConfig` (`options/bibliography.rs:19`, the runtime type
  engine code reads), and the hand-written
  `BibliographyOptions::to_bibliography_config()` conversion
  (`options/mod.rs:981`) — this last one is easy to miss since it isn't a
  derive; forgetting it silently drops an authored `online-access:` block
  before it reaches the engine.
- `medium_marker`/`cited_date_label` are `SubstituteMessage`
  (`options/substitute.rs:69`), a mapping shape (`{message: ...}`) with
  `deny_unknown_fields` — a bare string will not parse; don't propose a
  string-shorthand for these two fields.
- Anchor selection: `Reference::container_title().is_some()`
  (`crates/citum-schema-data/src/reference/accessors.rs:1108`). **Update
  (2026-09-08):** the legal-type edge case is resolved with a new engine
  predicate rather than a data-crate accessor —
  `container_title_is_embedded_work` (`processor/rendering/grouped/
  component_predicates.rs`) matches `reference.extension()` against the
  same embedded-`ClassExtension` set `values/title.rs`'s
  `resolve_primary_title` already uses for `ParentMonograph`/
  `ParentSerial` (`Monograph`/`CollectionComponent`/`SerialComponent`/
  `Serial`/`Event`/`AudioVisual`), excluding `LegalCase`/`Statute`/
  `Regulation`/`Treaty`. No data-crate change was needed:
  `citum_schema::reference::ClassExtension` and `Reference::extension()`
  are already public and already matched directly in engine code, so this
  stayed engine-local.
- The marker and cited-date bracket are injected by rewriting the
  resolved template (`Renderer::apply_online_access_bibliography_policy`,
  `processor/rendering/grouped/template_policy.rs`), the same mechanism
  `article-journal`/`anonymous-entries` bibliography policy already uses.
  A shared helper, `attach_after_first`, finds the first matching
  component (siblings first, then recursing into nested `Group`s — but
  **not** into `Message` args) and replaces it with
  `Group([original, addition], delimiter: Space)`, so the addition reads
  as part of the same "cell" regardless of the enclosing list's own
  delimiter. **Known gap:** NLM and CSE both nest their `chapter`
  type-variant's container title inside a `pattern.in-container-colon`
  message's `args` (not a plain `Group`), so `attach_after_first` — like
  the pre-existing `anonymous-entries` rewrite it's modeled on — does not
  find it there; chapter-type entries needing the marker on a
  message-args-nested container title don't get one. Not required by
  Acceptance Criteria below (the `report-core.js` diff shows 0
  regressions with this gap present); tracked for follow-up if a chapter
  row surfaces it.
- **The spec is silent on where the cited-date bracket attaches; resolved
  during implementation as immediately after the issued date**
  (`is_issued_date_component`), matching NLM's real macro nesting
  (`accessed-date` is called from inside the `date` macro, right after
  `date variable="issued"` —
  `taylor-and-francis-national-library-of-medicine.csl:164-198`). Applied
  uniformly to CSE too rather than replicating CSE's own, differently
  positioned `cited` macro call (after `container`, unrelated to
  `date: issued`) — simpler, and empirically zero-regression per the
  Acceptance Criteria's `report-core.js` diff.
- **Update (2026-09-08): this claim was wrong for `term.internet`.**
  `term.cited` and `term.accessed` resolve for free — both are named
  `GeneralTerm` variants, so the generic `term.X` message path falls back
  to the legacy `terms:` table's `cited`/`accessed` entries with no new
  locale content. `term.internet` does not: "internet" is not a
  recognized `GeneralTerm` variant (confirmed in
  `crates/citum-schema-style/src/locale/raw_conversion.rs`'s
  `parse_general_term`), so it falls into `GeneralTerm::Unknown` and never
  reaches the legacy `terms.internet` entry that already exists in
  `en-US.yaml` — it resolved to an empty string. Fixed by adding a direct
  `messages:` entry, `term.internet: "internet"`
  (`crates/citum-schema-style/embedded/locales/en-US.yaml`), text copied
  verbatim from the shipped CSL (`term="internet"`), same as
  `GROUP_SELECT.md`'s `term.place-unknown` precedent. Caught by a failing
  engine test before it reached a style file; would otherwise have
  silently rendered `[]` for every reference with a URL.
- **Update (2026-09-08): the fix above was itself incomplete — found by
  an adversarial review before merge.** Every non-en-US `Locale` is built
  via `from_raw_with_base(raw, Locale::en_us())`
  (`locale/raw_conversion.rs`): it starts as a full clone of en-US, then
  layers the locale's own `messages:`/`terms:` on top. Since no locale's
  own `messages:` defined `term.internet`, every locale's final `messages`
  map inherited the en-US entry above unshadowed — `de-DE`, `ja-JP`,
  `tr-TR`, `ko-KR`, `ru-RU`, `fr-FR` all render the English word
  "internet" despite each already carrying its own translated
  `terms: internet: {...}` entry, sitting unused for the same reason the
  original bug existed (not a recognized `GeneralTerm`). Fixed properly
  this time: added `GeneralTerm::Internet` to the enum
  (`locale/types.rs`), `"internet" => Some(GeneralTerm::Internet)` to
  `parse_general_term`, and `GeneralTerm::Internet => "internet"` to
  `general_term_to_message_id` (`locale/message_ids.rs`) — no new match
  arm needed in `general_term()`'s resolution logic itself, since the
  existing generic `self.terms.general.get(term)` early-return already
  handles any recognized variant, and the same base-then-overlay
  inheritance that caused the bug now gives correct graceful fallback
  (locales without their own `internet:` term, e.g. `ar-AR`/`es-ES`/
  `eu-ES`/`zh-CN`, inherit en-US's — locales with one get their own).
  Removed the `messages:` entry above, now unnecessary.
- **Found independently while fixing the above:**
  `BibliographyOptions::merge` (`options/mod.rs`) listed `article_journal`
  — `online_access`'s direct structural precedent — in its
  `merge_options!` call, but not `online_access` itself. A style
  inheriting an `online-access:` block from a base style through this
  typed merge path (not the raw-YAML cascade the embedded styles
  currently use) silently lost it. One-line fix; the round-trip test
  added above only checked direct conversion, not inheritance, so it
  didn't catch this.
- Independent of the `render-when` disposition: this is a new
  domain-specific bibliography option, not a candidate for `select: first`
  or `render-when`.

## Acceptance Criteria

- [x] `OnlineAccessConfig` added to `BibliographyOptions`,
      `BibliographyConfig`, and `to_bibliography_config()`; a round-trip
      test confirms an authored value reaches the runtime config.
      (`crates/citum-schema-style/src/options/bibliography.rs::tests::
      test_online_access_authored_value_reaches_runtime_config`.)
- [x] `medium_marker`/`cited_date_label` reject bare-scalar input.
      (`test_online_access_rejects_bare_scalar_medium_marker`.)
- [x] Anchor selection uses `container_title().is_some()`, with the
      legal-type edge case resolved and covered by a fixture confirming
      the marker lands on the reference's own title, not a container.
      (`crates/citum-engine/src/processor/rendering/tests.rs::
      online_access::medium_marker_attaches_to_own_title_for_a_legal_type_with_a_flat_reporter_container`,
      a native `Statute` reference with a `code` — verified to fail
      without the `container_title_is_embedded_work` check. Used a native
      construction rather than the `TLIB-SEL-BILL-1` fixture named in the
      original draft, per this repo's "no CSL-JSON in native tests"
      convention.)
- [x] Exact-output fixture per style confirms the accessed-date bracket
      text: `[cited …]` for NLM/springer, `[accessed …]` for CSE. Both
      terms verified via the engine test module below
      (`cited_date_bracket_renders_after_the_issued_date_when_url_and_accessed_are_present`
      for `term.cited`,
      `cited_date_bracket_uses_the_configured_label_term_not_a_hardcoded_one`
      for `term.accessed`); springer's is pre-existing (see Implementation
      Notes) and unaffected by this change.
- [x] All three embedded styles updated; `report-core.js --diff` shows the
      targeted `[Internet]`/`[cited …]`/`[accessed …]` rows flip with 0
      regressions. (The "Retrieved from"/"Accessed DATE"/bare-URL rows are
      `csl26-e8ul`'s scope, not this option's — don't count them here.)
      Measured per-entry (paired by fixture id, before vs. after), not
      just aggregates: 4 springer, 2 CSE, and 3 more across CSE/NLM (7
      distinct entries) flip from failing to passing, every one containing
      `[Internet]`, `[cited`, or `[accessed` — no unexplained flips. 0
      regressions anywhere in the 35-style corpus, confirmed stable after
      `cargo fmt`/`just schema-gen`. Aggregates:
      `springer-vancouver-brackets.exactParity` 49/67 → 53/67,
      `taylor-and-francis-council-of-science-editors-author-date.bibliography`
      44/47 → 46/47 (`.exactParity` 46/67 → 48/67),
      `taylor-and-francis-national-library-of-medicine.bibliography` 45/47
      → 46/47.
      **Update (2026-09-08, adversarial review):** the date-form and
      capitalization gaps noted below at ship time are now fixed. Added
      `DateForm::YearMonthAbbrDay` (year, abbreviated month with periods
      stripped, day — no comma) to reproduce CSL's `form="text"` shape
      exactly, wired for both NLM and CSE; forced `text_case: AsIs` on the
      injected cited-date label so it's never auto-capitalized. Verified
      exact per-entry text now matches oracle (e.g. `TLIB-SEL-MAP-1`:
      citum's cited-date fragment reads `[accessed 2019 Oct 27]`, byte-
      identical to oracle) — confirmed by direct CLI rendering, not just
      the aggregate `report-core.js` score, because that entry's
      `exactMatch` stays `false` for two *other*, unrelated, pre-existing
      reasons: CSE's cited-date bracket attaches after `date: issued` for
      every type (this spec's own documented, deliberate simplification —
      real CSL positions it differently, after `container`, for CSE's
      non-book/chapter branch specifically), and a publisher/place
      divergence unrelated to online-access entirely. Springer's `[Cited
      …]` observation at ship time was a **misattribution**: springer
      doesn't set `cited-date-label` (it already had its own, separate,
      pre-existing `pattern.cited-date`/`pattern.accessed-date` messages —
      see Implementation Notes), so its capitalization is a distinct,
      pre-existing bug in that unrelated code path, not a regression or
      gap in this option. Split out as `csl26-wt1u` (`csl26-zgdf` itself
      is now resolved).
      Regression tests: `en_us_year_month_abbr_day_strips_periods_and_omits_the_comma`
      and two more in `values::date`'s test module.
      Original ship-time note, for context: per-entry inspection of rows
      still failing (11 for NLM, 7 for springer, 6 for CSE) showed the
      `[Internet]` marker rendering correctly on every one — the remaining
      gap was entirely the `csl26-e8ul`-scoped "Available from:"/"Accessed
      DATE" access phrase, plus the date-form/capitalization issues now
      fixed above. No chapter-type row in the corpus exercises the
      message-args-nested-container gap noted in Implementation Notes, so
      that gap still has zero measured impact.
- [x] `just schema-gen` run, schema docs updated (`docs/schemas/style.json`
      only — no data-model reference changes, as expected for a
      style-schema-only option).
- [x] Status promoted to Active in the implementation commit.

New behavior tests (`crates/citum-engine/src/processor/rendering/tests.rs::online_access`,
7 tests): marker on own title with no container; no marker without a URL;
marker on an embedded container title; marker on own title for a legal
type despite a flat-string container; cited-date bracket after issued
date; cited-date bracket absent when the label isn't configured; cited-date
bracket uses the configured label term, not a hardcoded one.

Adversarial-review-fix tests: `locale::terms::tests::test_internet_general_term_resolves_a_locales_own_translation`
and `..._falls_back_to_the_base_locale_when_unset`
(`crates/citum-schema-style/src/locale/terms.rs`);
`options::bibliography::tests::test_online_access_survives_bibliography_options_merge_when_child_has_none`
and `..._child_override_wins_on_bibliography_options_merge`
(`crates/citum-schema-style/src/options/bibliography.rs`);
`values::date::locale_pattern_tests::en_us_year_month_abbr_day_*` (3 tests,
`crates/citum-engine/src/values/date.rs`), the middle one verified to fail
before the fix (asserting the old, wrong "2024, January 15" shape).

## Changelog

- v1.1 (2026-09-08): Implemented. Status promoted to Active. Corrected two
  Implementation Notes claims found during implementation: `term.internet`
  needed a new `messages:` entry (not already-resolving, unlike
  `term.cited`/`term.accessed`), and `cited-date-form: text` has no exact
  `DateForm` equivalent. An adversarial review of this same
  not-yet-merged implementation then found the `messages:` fix above was
  itself incomplete (every non-en-US locale rendered the English word
  "internet" despite already carrying its own translation — fixed
  properly with `GeneralTerm::Internet`, see the Implementation Notes
  update) and that `BibliographyOptions::merge` omitted `online_access`,
  silently dropping it on style inheritance; both folded in before
  merge, alongside the `DateForm::YearMonthAbbrDay` variant and a
  `text_case: AsIs` fix for the cited-date bracket's capitalization
  (see the Acceptance Criteria update). Resolved the legal-type anchor-selection gap with
  an engine-local predicate (no data-crate change needed). Recorded the
  cited-date bracket's attachment point (after `date: issued`), which the
  original draft left undecided.
- v1.0 (2026-09-08): Initial draft.
