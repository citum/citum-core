# Medium Designator Specification

**Status:** Draft
**Version:** 1.0
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
      cited-date-form: text
```

- `medium-marker`: locale message rendered bracketed and capitalized-first,
  anchored per "Anchor selection" below.
- `cited-date-label`: locale message naming the term inside the
  accessed-date bracket (see "Cited-date label" below). Omitting it
  disables the bracket even when a URL exists.
- `cited-date-form`: date form for the accessed-date bracket; only
  meaningful when `cited-date-label` is set.

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
  (`crates/citum-schema-data/src/reference/accessors.rs:1108`). The legal-type
  edge case above needs `container_title()` to also expose (or a second
  accessor to distinguish) whether the container came from an embedded
  `WorkRelation` versus a flat reporter/code field
  (`accessors.rs:1124-1127`, `ClassExtension::LegalCase`/`Statute`/
  `Regulation`/`Treaty`) — not yet decided which; either is a small,
  mechanical addition.
- No new locale term or override file is needed: `term.internet`,
  `term.cited`, and `term.accessed` all already exist in the base `en-US`
  locale, and both fields just reference them by name.
- Independent of the `render-when` disposition: this is a new
  domain-specific bibliography option, not a candidate for `select: first`
  or `render-when`.

## Acceptance Criteria

- [ ] `OnlineAccessConfig` added to `BibliographyOptions`,
      `BibliographyConfig`, and `to_bibliography_config()`; a round-trip
      test confirms an authored value reaches the runtime config.
- [ ] `medium_marker`/`cited_date_label` reject bare-scalar input.
- [ ] Anchor selection uses `container_title().is_some()`, with the
      legal-type edge case resolved and covered by a fixture (e.g.
      `TLIB-SEL-BILL-1`) confirming the marker lands on the reference's own
      title, not a container.
- [ ] Exact-output fixture per style confirms the accessed-date bracket
      text: `[cited …]` for NLM/springer, `[accessed …]` for CSE.
- [ ] All three embedded styles updated; `report-core.js --diff` shows the
      targeted `[Internet]`/`[cited …]`/`[accessed …]` rows flip with 0
      regressions. (The "Retrieved from"/"Accessed DATE"/bare-URL rows are
      `csl26-e8ul`'s scope, not this option's — don't count them here.)
- [ ] `just schema-gen` run, schema docs updated.
- [ ] Status promoted to Active in the implementation commit.

## Changelog

- v1.0 (2026-09-08): Initial draft.
