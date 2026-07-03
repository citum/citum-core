---
# csl26-q05f
title: Expose original-publication fields to render-when
status: in-progress
type: feature
priority: normal
created_at: 2026-07-02T23:42:51Z
updated_at: 2026-07-03T00:30:38Z
parent: csl26-h7oc
blocking:
    - csl26-giun
---

TemplateConditionField (crates/citum-schema-style/src/template.rs:1335) is a closed enum and does not expose original-date / original-publisher / original-publisher-place, so type-variants cannot gate on them with render-when. This blocks the CMOS 18 Reprint / 'Originally published as X (publisher)' trailer cluster deferred from the chicago-author-date bibliography cluster-lift PR (#996, bean csl26-giun) — one of the two deferred pieces accounting for most of the gap between 344/400 and the >=360/400 target.

Scope: add the original-publication fields to TemplateConditionField (plus schema regen via just schema-gen), wire engine condition evaluation, then use them in chicago-author-date-18th.yaml to render the reprint/original-publication trailers. Accessors already exist (original_date, original_publisher_str, original_publisher_place — see csl26-ifhx scrap notes). Note from #996: fixture edition free-text has bespoke oracle capitalization that does not reduce to a simple rule; expect per-item judgment when tuning the trailer text.

## Todo
- [x] Add original-publication variants to TemplateConditionField + schema regen
- [x] Engine: evaluate the new condition fields
- [x] Rust tests per CODING_STANDARDS conventions
- [x] Wire chicago-author-date-18th reprint/originally-published trailers; measure shared-corpus delta

2026-07-02 scoping correction (pre-implementation): TemplateConditionField already has OriginalPublished -> reference.original_date() (crates/citum-engine/src/processor/rendering/grouped/core.rs:91). The actual enum gap is original-publisher / original-publisher-place (and possibly original-title). Verify the render-side component support for those fields before assuming the condition enum is the only blocker.

## Summary of Changes (2026-07-03)

Implemented on branch feat/original-publication-render-when, stacked on
fix/chicago-author-date-bib-clusters (#996).

**Layer 1 — condition enum** (crates/citum-schema-style/src/template.rs):
added `TemplateConditionField::{OriginalPublisher, OriginalPublisherPlace,
OriginalTitle}`. `OriginalPublished` (date) already existed per the
pre-implementation scoping correction.

**Layer 2 — engine evaluation** (crates/citum-engine/src/processor/rendering/
grouped/core.rs `condition_field_present`): wired the three new fields to
the existing `original_publisher_str` / `original_publisher_place` /
`original_title` accessors.

**Layer 3 — render side (the real gap)**: original-publisher and
original-publisher-place were already renderable via
`SimpleVariable::{OriginalPublisher, OriginalPublisherPlace}` (pre-existing,
unused). original-title had no render path: added
`TitleType::Original` + `resolve_primary_title` support
(crates/citum-engine/src/values/title.rs). Also found and fixed a real
adjacent gap while wiring the reprint trailer: `number:` components
(e.g. `edition`) silently ignored `text-case` overrides, unlike
variable/title/term components — fixed in
crates/citum-engine/src/values/number.rs (mirrors the existing
variable.rs pattern). Both are exercised by new integration tests in
crates/citum-engine/tests/bibliography.rs (rstest given/when/then for
the condition fields and the text-case fix, plus a single-scenario test
for TitleType::Original).

**Layer 4 — YAML** (chicago-author-date-18th.yaml `book` type-variant):
added an original-publisher/-place mid-clause (suppressed when
original-title is present), an unconditional `number: edition` render,
and a trailing "Originally published as {original-title} ({original-
publisher-or-place})" group gated on original-title presence.

Schema regenerated (`docs/schemas/style.json`, `server.json`).
`just pre-commit` equivalent (fmt/clippy/nextest, no `just` binary
available in this environment) green: 1713/1713.

**Oracle** (`chicago-18th.json`, --case-sensitive): citations stayed
15/15. Bibliography stayed 344/400 — no regressions, but no net
pass-count gain either. Ogawa (original-title case) is now a byte-exact
match (previously passed only via fuzzy similarity, missing the whole
trailer sentence). Ellison/Roy/Schweitzer (original-publisher, no
edition) now render the correct original-publisher clause too (still
pass, higher-fidelity text, missing only the literal "Reprint" word,
deliberately not implemented — see below).

**Why bibliography didn't net-improve**: the only 7 refs in the fixture
carrying original-publisher/-place/-title are Ogawa, Ellison, Roy,
Schweitzer (already covered, described above), plus three that remain
failing for reasons outside this bean's scope:
- fitzgeraldGreatGatsby1992 / emersonNature1985 (book, `edition` set):
  content is now 100% correct except that citeproc-js capitalizes one
  word mid-clause ("and Notes...", "with An...") in a way that does not
  reduce to any CSL-declared rule (traced into citeproc-js's
  `capitalizeFirst` — confirmed against the vendored citeproc.js: it
  should only capitalize the first word of the whole string, so this is
  a citeproc-js-internal quirk, not a style or citum defect). Matches
  the #996-inherited caveat exactly; not chased further.
- dindorfScholiaGraecaHomeri1962 (`classic`, no author, editor only):
  citeproc-js leads with title (not the substituted editor) for
  editor-only classic works — an unrelated author-substitute/ordering
  gap in the (currently variant-less, default-template) `classic` type,
  not an original-publication field gap. Out of scope; would need its
  own type-variant work.
- langeBlackMariaOakland1965 (`graphic`, no book variant at all): needs
  medium/dimensions/archive/status/url support, unrelated to this bean.

Deliberately not implemented: the literal "Reprint, {publisher}" word
(CSL's `source-publication-description-bib` "reprint" text) — doing so
correctly requires distinguishing "edition present" from "edition
absent" cases (CSL renders edition text OR the literal word, never
both), which would need a `TemplateConditionField::Edition` addition
beyond this bean's stated scope. Since every refs in this fixture with
original-publisher-or-place already has a working, correct output
(either edition text or a plain publisher join), skipping the literal
word costs nothing in this corpus and was left out to keep the change
minimal.

**report-core.js**: `chicago-author-date-18th` fidelity 0.898 (baseline
0.898, unchanged), quality/SQI 0.921 (baseline 0.925, -0.004) — the dip
is entirely the `concision` subscore (structurally-necessary
single-item `group: [...]` wrappers needed to attach `render-when`,
since conditions can only be set on TemplateGroup, not bare
components); `withinScopeDuplicates` rose 39->45. No other subscore
moved. `taylor-and-francis-chicago-author-date`: fidelity 0.898,
quality 1.0 intact (hard requirement met), bibliography 441/498
unchanged — correctly inherits the parent's (unchanged) pass count.

Bean left in-progress per instructions: final report-core sweep and PR
are the supervisor's after #996 merges.
