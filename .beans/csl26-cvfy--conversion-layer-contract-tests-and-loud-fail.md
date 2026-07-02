---
# csl26-cvfy
title: CSL-JSON conversion-layer contract tests + loud-fail on unmapped types
status: in-progress
type: epic
priority: medium
created_at: 2026-07-02T00:00:00Z
updated_at: 2026-07-02T17:22:41Z
---

## Problem

Style fidelity is measured by running raw CSL-JSON fixtures through the
*entire* pipeline — CSL-JSON parsing → note-field-hack extraction →
CSL-type routing → `InputReference` → style template rendering — and
diffing the final citation text against citeproc-js
(`node scripts/oracle.js`, `node scripts/report-core.js`). A single
pass/fail number comes out. When it fails, nothing tells you *which layer*
broke: style-authoring bug, CSL-type-routing bug, or CSL-JSON parsing
quirk. Today's session spent real wall-clock time doing this attribution
by hand for one failure (root-causing `chi-manuscript` in the
`chicago-shared-corpus` fixture, see `csl26-shco`):

1. Ran `node scripts/oracle.js styles-legacy/chicago-notes.csl
   --refs-fixture tests/fixtures/test-items-library/chicago-18th.json
   --citations-fixture .../chicago-18th-citations.json --json` and got a
   garbled citation with no clue why.
2. Isolated the single reference into its own fixture file and rendered it
   directly via `cargo run --bin citum -- render refs -b <item> -s
   <style> -m cite --json` to reproduce in isolation.
3. Injected a temporary marker string into the style YAML's
   `manuscript:` type-variant to *empirically* prove the type-variant was
   never being selected at all (it wasn't — the marker never appeared).
4. Ran `cargo convert refs <item> --from csl-json -o <out>.yaml` to inspect
   the intermediate `InputReference` and found `type: document` instead of
   the expected `type: manuscript`.
5. Traced through `crates/csl-legacy/src/csl_json.rs` (note-field hack
   parser) → `crates/citum-schema-data/src/reference/conversion/mod.rs`
   (CSL-type routing switch) → `conversion/scholarly.rs` to find the root
   cause: the fixture's `note` field carries a Zotero/Better-BibTeX
   type-override (`"type: collection"`), which
   `Reference::parse_note_field_hacks` (`csl_json.rs:413-415`)
   unconditionally applies over the top-level `"type": "manuscript"`, and
   `"collection"` has **no arm at all** in the routing `match` in
   `conversion/mod.rs:346-369` — so it silently falls through to a
   generic default.

That is five tool invocations and four source files to attribute *one* of
eight known-failing fixture cases in `chicago-shared-corpus` alone. This
does not scale, and it will recur every time a `tune` pass touches a style
whose fixture references exercise an edge case in the conversion layer.

## Root causes (concrete, file:line)

1. **The CSL-type routing table is hand-maintained and silently
   incomplete.** `crates/citum-schema-data/src/reference/conversion/mod.rs`,
   the `match legacy.ref_type.as_str()` block starting at line 346, has no
   exhaustiveness check against CSL 1.0's actual type vocabulary. An
   unmapped `ref_type` string (e.g. `"collection"`) falls through to
   whatever the final wildcard arm does (verify current behavior — as of
   this writing it lands on a generic monograph/document shape with most
   fields unset).
2. **The note-field type-override is unvalidated.**
   `crates/csl-legacy/src/csl_json.rs:413-415`
   (`Reference::parse_note_field_hacks`) does
   `self.ref_type = value.to_string();` for any `note:` line matching
   `type: X`, with no check that `X` is a recognized CSL type. A typo or
   copy-pasted artifact in a Zotero export silently reclassifies the
   reference.
3. **Prior art exists for the "loud fail" pattern, but at the wrong
   layer.** `crates/citum-schema-data/src/reference/accessors.rs:1462-1478`
   (the `ClassExtension::Unknown` arm of `ref_type()`) already has a
   `debug_assert!` plus a `TODO(csl26-1bdr)` gesturing at
   `CompatibilityWarning` plumbing for *unknown top-level classes*
   (`class:` field, e.g. `monograph`/`serial`/`legal-case`) — see the
   archived bean `csl26-1bdr` and
   `docs/specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md`. That mechanism is
   one level too coarse: it only fires for a completely unrecognized
   `class`, not for a recognized class (`monograph`) whose legacy CSL
   `type` string (`collection`) doesn't map to any of that class's known
   type variants. This bean is the analogous fix one level down — do not
   conflate the two; check whether the existing
   `ReferenceClass::KNOWN` const (`crates/citum-schema-data/src/reference/classes.rs:76`)
   and its `discriminator_tests.rs` fuzz coverage suggest a pattern worth
   mirroring for per-class type vocabularies.
4. **Failure attribution is entirely manual.** `scripts/oracle.js`'s
   `renderWithCitumProcessor` (line 476) shells out to
   `cargo run --bin citum -- render refs` — the *exact same* CLI path used
   for the manual investigation above. This means any fix to the
   conversion layer is immediately visible to oracle.js with no separate
   JS-side change required, and it also means oracle.js is well
   positioned to get per-reference conversion diagnostics for free if the
   CLI exposes them (see Phase 4 below).

## Proposed pivot

Decouple "does this CSL-JSON reference convert to a sane `InputReference`"
from "does this style render the correct text" — make the former a fast,
targeted, independently-owned test surface that fails at the point of the
actual defect, not three layers downstream in a citation-text diff.

### Phase 1 — Establish the CSL 1.0 type vocabulary as a source of truth
Determine whether `csl-legacy` (or anywhere else in the workspace) already
has a canonical list of CSL 1.0 `type` strings. If not, author one (likely
as a Rust `const` slice in `crates/csl-legacy/src` alongside — or feeding
— the routing table), sourced from the CSL 1.0 spec's `csl-types.rnc` /
official type list, not reverse-engineered from what happens to appear in
fixtures today. This must include less-common types the current routing
table has no arm for (audit needed — `collection` was found by accident,
there are likely others: e.g. `figure`, `graphic`, `map`, `musical_score`,
`performance`, `review`, `review-book`, `song`, `standard`, `treaty`, verify
each against the current `match` arms in `conversion/mod.rs`).

### Phase 2 — Exhaustiveness test at the conversion layer
Add a test (likely in
`crates/citum-schema-data/src/reference/conversion/tests.rs` or a new
sibling module) that iterates the Phase 1 vocabulary and asserts each
value produces a `ref_type()` round-trip that is *not* a generic/default
fallback — i.e., converting `"type": "collection"` through
`InputReference::from` and back through `.ref_type()` must not silently
collapse into `"document"` or another type the fixture didn't ask for.
This is the test that should have caught the `chi-manuscript` case; write
it first as a red test reproducing that exact regression before touching
`conversion/mod.rs`.

### Phase 3 — Validate the note-field type-override
In `Reference::parse_note_field_hacks`
(`crates/csl-legacy/src/csl_json.rs:413-415`), check the override value
against the Phase 1 vocabulary before assigning `self.ref_type`. Decide
(and document, this needs a product decision, not just an engineering
one) what happens on an unrecognized override: ignore it and keep the
top-level type, or apply it anyway but flag a compatibility warning. Given
the `chi-manuscript` fixture item apparently *wants* `"collection"` as its
true type (not just a bad override), this phase depends on Phase 4's
routing fix landing first, or the "ignore unrecognized override" policy
would incorrectly suppress a legitimate CSL type this fixture is
exercising.

### Phase 4 — Close routing gaps + make fallthrough loud
For every Phase 1 vocabulary entry with no arm in `conversion/mod.rs`,
either add a real routing arm (preferred, if a sensible `ClassExtension`
mapping exists — `collection` likely wants its own arm rather than
defaulting into `Monograph`/`Document`) or make the fallback path loud:
a `debug_assert!` plus `CompatibilityWarning` plumbing analogous to the
existing unknown-*class* mechanism at `accessors.rs:1462-1478`, but scoped
to unknown-*type-within-known-class*. Coordinate with whatever comes out
of the still-pending `csl26-1bdr` follow-on work so the two loud-fail
mechanisms share a design rather than diverging.

### Phase 5 — Attributed fidelity reporting (stretch goal, separate PR)
Once Phase 2-4 land, consider exposing per-reference conversion
diagnostics from `citum render refs` (e.g. a `--json` field noting
"rendered via fallback/default type") so `scripts/oracle.js` and
`scripts/report-core.js` can tag a failing fixture case as
"conversion-layer suspect" *before* it's counted as a style-fidelity
failure, instead of requiring the manual trace this bean's problem
statement describes. This directly mechanizes the classification rule
already written in `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md`
(style-defect / migration-artifact / processor-defect / intentional
divergence) instead of leaving it as pure human/LLM judgment every time.

## Non-goals
- Do not fix `chicago-shared-corpus`'s remaining 8 fixture failures as
  part of this epic — that's `csl26-shco`'s scope, and several of those
  failures may turn out to be genuine style-defects once conversion-layer
  noise is removed. This epic is about the *testing architecture*, not
  about driving any one style to 100%.
- Do not expand this into a general InputReference schema redesign — the
  archived `csl26-1bdr` already covers the top-level `class:` discriminator
  question and is a settled, separate concern (pre-1.0 major-bump
  opt-out). Stay scoped to the CSL-legacy-`type`-string → Citum-type
  mapping completeness problem.

## Spec requirement
Per `CLAUDE.md`'s Documentation Rule, author a spec in `docs/specs/`
(status `Draft` → `Active` in the implementation commit) before
implementing Phase 2 onward — this bean and its problem statement are
sufficient input for that spec's first draft, but the spec itself is not
written yet.

## Todo
- [x] Author `docs/specs/CSL_TYPE_CONVERSION_CONTRACT.md` (or similarly
      named) — Draft status, scoping Phases 1-4 as the MVP and Phase 5 as
      a stretch/follow-on
- [x] Phase 1: source-of-truth CSL 1.0 type vocabulary + gap audit against
      current `conversion/mod.rs` routing arms
- [x] Phase 2: red test reproducing the `chi-manuscript`/`collection`
      regression at the conversion layer, independent of any style
- [x] Phase 3: validate note-field type-override against the vocabulary
      (sequence after Phase 4's routing fix per the dependency noted above)
- [x] Phase 4: close routing gaps; design + implement loud-fail for
      unmapped types, coordinated with `csl26-1bdr`'s prior art
- [ ] Phase 5 (stretch, separate PR): attributed fidelity reporting in
      `oracle.js`/`report-core.js`
- [x] Re-run `node scripts/report-core.js --style chicago-notes-18th`
      after Phase 4 lands to confirm `chi-manuscript` (and any other
      newly-routed types in the shared corpus) improve without YAML
      changes, validating the "conversion bugs, not style bugs" diagnosis
      in `csl26-shco`. **Outcome:** the conversion layer is fixed and
      verified (`chi-manuscript` converts to `ref_type() == "collection"`
      with `archive-info` intact; contract test + CLI trace both green),
      the full corpus shows zero style regressions and a small
      bibliography improvement (7790→7792 passed), but the shared-corpus
      citation itself still fails (7/15 unchanged) because the *migrated
      style* has no `collection` type-variant — the failure is now
      cleanly attributable to the style/migration layer, which is
      exactly `csl26-shco` scope. Diagnosis mechanism validated; the
      "without YAML changes" improvement expectation was optimistic.

## Summary of Changes

Implemented in PR #993 (branch `epic/csl-json-conversion-contract-tests`):

- `docs/specs/CSL_TYPE_CONVERSION_CONTRACT.md` (Active): vocabulary
  source of truth, canonicalization table, routing closure design,
  loud-fail pattern, note-override policy.
- `crates/csl-legacy/src/csl_json.rs`: `CSL_TYPES` (45 CSL 1.0.2 types)
  + `CSL_TYPE_EXTENSIONS`; `parse_note_field_hacks` now applies a
  `type: X` override only for recognized types — unrecognized values
  keep the top-level type and stay in the note.
- `crates/citum-schema-data/src/reference/conversion/`: 12 routing gaps
  closed (11 anticipated + test-discovered `post-weblog`→`post`
  collapse; `speech`→`event` collapse also fixed). CSL `collection`
  routes as an *archival* document shape (genre-discriminated) because
  Citum's editorial `Collection` class has no author/archive fields —
  recorded in the spec. Wildcard fallback now `debug_assert!`s on known
  CSL types (mirrors `accessors.rs` `TODO(csl26-1bdr)` prior art).
- `conversion/contract_tests.rs`: expectation-table round-trip test over
  all 45 types (zero exceptions), chi-manuscript regression test with
  archive-preservation assertion, capitalized-genre (`Map`) regression
  test.
- Fidelity: zero regressions across the core corpus; bibliography
  passed 7790→7792; chicago-author-date-18th and
  taylor-and-francis-chicago-author-date +0.002 each.

Phase 5 filed as `csl26-3r34`. Remaining `chi-manuscript` citation
mismatch reclassified as a style/migration-layer defect (`csl26-shco`).
