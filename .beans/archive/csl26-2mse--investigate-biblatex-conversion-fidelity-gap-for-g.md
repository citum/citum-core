---
# csl26-2mse
title: Investigate BibLaTeX conversion fidelity gap for GB/T corpus
status: completed
type: task
priority: normal
created_at: 2026-07-23T20:55:21Z
updated_at: 2026-07-24T11:28:49Z
---

On the gb7714-bench benchmark, citum's .bib source path (via citum convert refs, BibLaTeX->Citum-YAML) diverges from Zotero far more than the .json path: even with the terminal-period fix applied as a counterfactual, exact-match-vs-Zotero stayed at 12-14% on builtin.bib/better.bib vs 76-88% on builtin.json/better.json (same underlying references, same rendering style). This points to a real gap in citum-migrate's BibLaTeX field mapping/conversion, separate from bibliography-template formatting. See docs/architecture/audits/2026-07-23_GB7714_BENCH_COMPARISON.md 'Scope note' section. Needs its own investigation to characterize which fields/entry types are mismapped.

## Summary of Changes

Investigated citum-refs/src/biblatex.rs (the actual BibLaTeX->Citum conversion
code -- this bean's body and the parent audit's "Scope note" misattribute it
to citum-migrate, which only does CSL 1.0 style-XML migration, unrelated).

Fixed on branch `fix/csl26-2mse-biblatex-type-mapping`:

- **Entry-type mapping**: `techreport`/`thesis`/`phdthesis`/`mastersthesis`/
  `online`/`unpublished`/`proceedings`/`mvproceedings` now map to their
  correct `MonographType` (Report/Thesis/Webpage/Manuscript/Book) instead of
  falling to generic `Document`. Confirmed against the real gb7714-bench
  corpus (`~/Code/gb7714-bench/data/data/`, pinned rev `42e5c083`, not
  committed to citum-core): reclassifies 38/344 entries in `builtin.bib` and
  40/344 in `better.bib` that previously collapsed to `document`
  (32.8%/37.2% -> 21.8%/25.6%).
- **Field-mapping fixes**, all targeting fields that already exist on the
  Citum schema (checked directly against `citum-schema-data`, not assumed):
  `translator` (was hardcoded `None` in all 3 builders despite a typed
  accessor existing on the `biblatex` crate), publisher fallback to
  `institution`/`organization`/`school` when `publisher` is absent (fixes a
  real bug where `location` was also silently dropped whenever an entry used
  `institution` instead of `publisher` -- confirmed on real corpus entry
  `gbt7714.7.5.1:3`), `subtitle` -> `Title::Structured`, `abstract` ->
  `abstract_text`, `version`, `keywords` (Monograph only), and ISBN
  propagated to the synthesized parent `Collection` for
  `inbook`/`incollection`/`inproceedings`.
- 6 new tests added to `crates/citum-refs/src/biblatex.rs`'s test module
  (typed-field assertions, no substring `contains()`).

Deferred to `csl26-11h2` (one consolidated bean, not several): editor
sub-role differentiation, `eprint`/`eprinttype` preprint detection, `series`
modeling, remaining entry types with no `InputReference` target yet (patent/
dataset/software/standard/map/archive/periodical/reference variants --
`specialized.rs` already has standalone Patent/Dataset/Standard/Software
classes, so this is new builder functions, not a schema change),
`eventtitle`/`venue`/`chapter`, a `.bib` fixture corpus + contract tests, and
a `field_str` typed-accessor refactor. Each of those needs a decision only
Bruce can make (new enum variant, modeling choice, or separately-scoped
effort) -- everything that was just "read an existing field" got fixed in
this pass instead.
