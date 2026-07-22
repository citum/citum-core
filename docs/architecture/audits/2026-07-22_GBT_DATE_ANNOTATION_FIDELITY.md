# GB/T 7714 Date-Annotation Fidelity — Fix Record

- **Date:** 2026-07-22
- **Bean:** `csl26-d3hs`
- **Spec:** [docs/specs/MULTILINGUAL.md](../../specs/MULTILINGUAL.md), [CALENDAR_DATE_ANNOTATIONS.md](../../guides/AUTHORING_LOCALES.md)

## Problem

`docs/compat.html` reported GB/T 7714—2025 numeric bibliography fidelity as "146/203"
with no further context, while an earlier session had claimed "100% (adjusted)" —
apparently contradictory. Neither number was wrong on its own terms: 146/203 is the
*raw* citeproc-js byte-parity count (recorded in `csl26-d3hs`'s 2026-07-21 update);
`docs/compat.html` itself was stale and contained no GB/T rows at all (generated before
the styles were embedded). Reconciling the two required determining, for each of the
11 items still failing after the `div-010` Latin-punctuation adjustment, whether Citum
or the citeproc-js/CSL-M oracle was actually correct.

## Root cause

8 of the 11 remaining mismatches are historical or uncertain publication dates where
Citum renders an author-supplied CSL cheater-syntax `note:` field `issued:` override
verbatim — GB/T 7714—2025 era-year parentheticals (§7.5.4.1, e.g. `1947（民国三十六年）`),
copyright/printing-year and approximate-year brackets (§7.5.4.3, e.g. `1995印刷`,
`[1936]`), and an unbracketed open-ended date range (§8.4.2, `1957—1990`). citeproc-js
either drops the annotation entirely or, for the open-ended range, adds a spurious
uncertainty bracket (`[1990]`) the source style does not justify.

The remaining 3 (Sagan/Bengio/Brown items) are a real but out-of-corpus gap: types
`broadcast`, `interview`, and `legal_case` do not appear anywhere in the 203-item GB/T
corpus (`tests/fixtures/test-items-library/gb-t-7714-2025.json`) and only surface
through the generic 47-ref default fixture `report-core.js` merges in for its
250-total headline. gb7714-bench and native reviewers exercise the 203-item corpus
only, so these do not affect the number that matters for the news post; deprioritized
to a future robustness pass rather than fixed here.

## Verification authority

`gb7714-bench` (the [gb7714-bench PR #25](https://github.com/YDX-2147483647/gb7714-bench/pull/25)
this session was prompted by) turned out to have no gold-string oracle of its own — it's
a cross-engine visual comparison (`/converge/`, `/compare/`, `/entry/` views), not a
pass/fail test suite. Its README instead points to
[`data/GB-T_7714—2025.original.toml`](https://github.com/typst-doc-cn/bib-csl-dev-data/blob/main/data/GB-T_7714—2025.original.toml)
in the `data` submodule it and the shared Zotero Chinese CSL corpus both reference —
text extracted directly from the official GB/T 7714—2025 standard PDF, organized by
the same `id-prefix` section keys (e.g. `gbt7714.7.5.4.1:`) as Citum's own fixture IDs.

All 8 disputed items were checked against the standard's own worked examples:

| Fixture item | Standard's own example (§) | Citum | citeproc-js/oracle |
|---|---|---|---|
| `gbt7714.7.5.4.1:1` | `1947 (民国三十六年)` | matches | drops annotation |
| `gbt7714.7.5.4.1:2` | `1705 (康熙四十四年)` | matches | drops annotation |
| `gbt7714.7.5.4.3:2` | `1995印刷` | matches | drops annotation |
| `gbt7714.7.5.4.3:3` | `[1936]` | matches | drops brackets |
| `gbt7714.8.2.2:2` | `1865 (清同治四年)` | matches | drops annotation |
| `gbt7714.8.4.2:2` | `1957—1990` (no brackets) | matches | adds spurious `[1990]` |
| `gbt7714.8.12.3:1` | `1887 (光绪十三年三月十三日)` | matches | drops annotation |
| `gbt7714.8.12.3:3` | `1949(中华民国三十八年八月)` | matches | drops annotation |

Every case: Citum's rendering matches the standard's own text exactly (modulo the
already-registered `div-010` full-width/half-width convention); citeproc-js is the one
that diverges from the standard it purports to implement.

## Fix

Registered **div-011** in `scripts/lib/oracle-divergences.js` (detector
`explainBibliographyMismatchFromDiv011`) and `scripts/report-data/verification-policy.yaml`.
Masks a bibliography mismatch only when (a) the item's `note` field carries an
`issued:` cheater-syntax override — scoping the mask to exactly the ~15 items in the
corpus authored this way, never an unrelated date mismatch — and (b) stripping the
recognized GB/T date-annotation forms (era parentheticals, approximate-year brackets,
the `印刷` suffix) from both citeproc's and Citum's text makes them equal. The
strip-then-compare approach handles both directions symmetrically: Citum-has-more
(7 cases) and citeproc-has-more (`8.4.2:2`'s spurious bracket) are the same check.

As a byproduct, discovered and fixed a **stale-cache bug** in `report-core.js`: its
benchmark-run cache key hashed `oracle.js` itself but not `oracle-divergences.js` or
`verification-policy.yaml`, so editing either silently kept serving pre-fix results —
this cost real debugging time mid-session (div-011 initially appeared to have zero
effect until the cache was manually cleared). Added `oracleDivergenceDepsHash()`,
folded into all three `runCachedJsonJob` call sites that shell out to `oracle.js`.

`gb-t-7714-2025-note` inherits the same shared-base date handling and reached 100%
adjusted fidelity on the 203-item corpus for free; its verification-policy benchmark
run was flipped from diagnostic-only (`count_toward_fidelity: false`) to gating
(`true`, `min_pass_rate: 1.0`).

`gb-t-7714-2025-author-date` was investigated but not fixed this session — root-caused
to two distinct, well-understood, but sizable mechanical bugs in its own
`bibliography.type-variants` (key-string mismatches against the shared base's grouping
that leave several of its own overrides silently shadowed, plus missing delimiters on
the overrides that do fire); full findings and a concrete fix recipe are recorded on
`csl26-6eak`.

## Verification

- `node scripts/oracle.js tests/fixtures/csl-m/gb-t-7714-2025-numeric.csl --json
  --scope bibliography --refs-fixture tests/fixtures/test-items-library/gb-t-7714-2025.json
  --case-insensitive`: adjusted bibliography 203/203 (was 195/203 before div-011,
  146/203 raw).
- Same for `gb-t-7714-2025-note.csl`: adjusted 203/203 bibliography, 1/1 citation
  sequence.
- `node scripts/report-core.js --style gb-t-7714-2025-numeric`: `benchmarkRunResults[0]
  .status: "pass"`, `fidelityScore: 0.989` (merged 250-ref report; the 203-corpus-scoped
  benchmark run itself is 100% adjusted — the 3 out-of-corpus type gaps are what keep
  the merged score below 1.0).
- `node --test scripts/oracle.test.js scripts/report-core.test.js`: 93 passed, including
  6 new div-011 unit tests (era-annotation mask, open-range spurious-bracket mask,
  no-override non-mask, non-annotation-mismatch non-mask, plus a
  `stripGbtDateAnnotations` unit test).
- `node scripts/report-core.js --write-html`: full regeneration confirms GB/T rows now
  appear in `docs/compat.html` with the corrected adjusted numbers (numeric 98.9%/100%
  badge, note 97.5%/100% badge — both "pass" on their 203-corpus benchmark gate;
  author-date unchanged at 54.4%, honestly reflecting the deferred work).
