---
# csl26-wt1u
title: Springer's own pattern.cited-date/pattern.accessed-date bracket has date-form and capitalization bugs
status: completed
type: bug
priority: low
tags:
    - style
    - fidelity
created_at: 2026-09-08T23:50:05Z
updated_at: 2026-09-15T15:16:16Z
parent: csl26-ccdt
---

springer-vancouver-brackets-core.yaml's pre-existing pattern.cited-date/pattern.accessed-date messages (unrelated to docs/specs/MEDIUM_DESIGNATOR.md's online-access option, which springer doesn't use for this bracket) render '[Cited 2024, January 15]' where oracle expects '[cited 2024 Jan 15]' -- wrong capitalization, wrong date form (long month + comma instead of abbreviated no-comma), and a stray period before the bracket ('2023.' vs '2023'). Found via report-core.js per-entry inspection while verifying an adversarial-review fix to the (separate) online-access cited-date bracket for NLM/CSE -- confirmed this is Springer's own, independent, pre-existing code path (this style has never set online-access.cited-date-label). Same underlying date-form gap as MEDIUM_DESIGNATOR.md's now-fixed DateForm::YearMonthAbbrDay -- likely fixable by migrating pattern.cited-date/pattern.accessed-date onto that same DateForm, but the capitalization/period issues need their own investigation.

## Summary of Changes

Fixed in place at `springer-vancouver-brackets-core.yaml`'s four hand-rolled `pattern.cited-date` sites (`webpage`, `entry-dictionary`, `map`, `standard`), rather than switching onto the `online-access` option — investigated that first and rejected it (see `docs/specs/MEDIUM_DESIGNATOR.md` v1.2's new "Known limitation" section: the option has no per-type exclusion, and Citum's CSL-type conversion collapses `bill`/`hearing` and `legislation`/`statute` into single Citum types, so there's no correct on/off answer for those two type-variants under a style-wide switch).

Three fixes, all confirmed against oracle:
- **Date form:** `year-month-day` → `year-month-abbr-day` (long month + comma → abbreviated, no comma).
- **Punctuation:** grouped `date: issued` + the message with `delimiter: " "` at `webpage`/`map`/`standard` (removes the stray `. ` the bibliography-wide separator was inserting between them as two separate top-level components). `entry-dictionary` has no `issued` anchor, so only the form fix applies there; `dataset`'s `extends: webpage` insertion point moved from `before: date: issued` (no longer a top-level match once grouped) to the equivalent `after: title: primary`.
- **Capitalization:** found and fixed a real engine bug along the way — `sentence_initial.rs`'s ambient capitalize-first pass for `Message` components ignored the component's own explicit `text_case`, so `text-case: as-is` on `entry-dictionary`'s message (the one site with no anchor, so it's genuinely sentence-initial) was silently overridden, still rendering "Cited". Fixed with a narrow guard (`!matches!(text_case, Some(AsIs | Lowercase))`), mirroring the existing `Contributor` arm's bypass precedent in the same file. Test: `sentence_initial_message_respects_an_explicit_as_is_or_lowercase_case` (`citum-engine/src/processor/rendering/tests.rs`). Confirmed no other embedded style has a `message:` component setting `as-is`/`lowercase` sentence-initial, so this is a no-op everywhere else in the corpus.

Unresolved, left as-is (not a new bean): `standard`'s hand-rolled bracket still fires on any `accessed` date regardless of URL presence — the CSL gates on URL specifically, but the frozen `render-when` vocabulary has no URL-presence field to express that gate. Pre-existing, not introduced by this fix.

Verification: `just pre-commit` (2807/2807 tests, fmt, clippy clean), `just oracle styles-legacy/springer-vancouver-brackets.csl` (47/47 bibliography, 20/20 citations — unchanged), full `report-core.js` run with no other embedded style affected.
