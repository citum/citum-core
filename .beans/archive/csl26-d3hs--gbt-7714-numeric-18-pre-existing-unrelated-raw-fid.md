---
# csl26-d3hs
title: 'GB/T 7714 numeric: 18 pre-existing unrelated raw fidelity failures'
status: completed
type: bug
priority: normal
tags:
    - fidelity
    - style
    - gb-t
created_at: 2026-07-17T22:53:37Z
updated_at: 2026-07-23T00:23:45Z
blocked_by:
    - csl26-7hsx
---

node scripts/oracle.js tests/fixtures/csl-m/gb-t-7714-2025-numeric.csl --json --scope both --refs-fixture tests/fixtures/test-items-library/gb-t-7714-2025.json --citations-fixture tests/fixtures/test-items-library/gb-t-7714-2025-numeric-citations.json --case-insensitive

shows 18 unmasked raw bibliography mismatches (of 203 refs) unrelated to punctuation, discovered while verifying csl26-fn9x (GB/T Latin-script punctuation fix). These keep gb-t-7714-2025-numeric's `min_pass_rate: 1.0` gate at `fail` independent of that fix — the gate was already failing before csl26-fn9x touched anything (85.2% raw / 85.7% adjusted at baseline).

Example: `gbt7714.7.1.3:2` (anonymous-author periodical, "Coffee drinking and cancer of the pancreas") is missing its year and issue components entirely, with wrong component ordering:

```
oracle: [21]Coffee drinking and cancer of the pancreas[J]. Br Med J，1981，283（6292）：628
citum:  [21]283Coffee drinking and cancer of the pancreas1981628
```

Looks like a substitution/anonymous-author-fallback template gap in the numeric type-variant for periodicals, not a punctuation issue.

Full list of affected ids (from baseline, still failing after adjustment):
gbt7714.7.1.3:2, gbt7714.7.2.1:7, gbt7714.7.2.3:7, gbt7714.8.11.3.2:5, gbt7714.8.14.3:3, gbt7714.8.15.2:3, gbt7714.8.1:4, gbt7714.8.4.2:4, gbt7714.8.5.3:8, gbt7714.8.5.3:9, gbt7714.8.6.1:5, gbt7714.8.6.3:2, gbt7714.8.8.3:4, gbt7714.8.9.2:4, gbt7714.9.2.1.3:1, gbt7714.9.2.2:6, gbt7714.9.3.1.1:2, gbt7714.9.3.1.2:1

- [x] Triage each of the 18 ids by root cause (anonymous-author substitution, missing components, ordering) — see "Update (2026-07-21)": root cause was resolve_localized_type_variant's missing fallback tier (csl26-7hsx), not anonymous-author substitution as originally guessed
- [x] Fix the underlying template/engine gaps — fixed by csl26-7hsx (locale-type-variant fallback); remaining divergence on 14/18 ids is the pre-existing, already-registered Latin-script punctuation convention, not a new gap
- [x] Confirm gb-t-7714-2025-numeric reaches its declared min_pass_rate: 1.0 gate — see "Numeric result": benchmarkRunResults status=pass, 100% adjusted fidelity on the 203-item corpus

## Update (2026-07-21)

Root cause identified and fixed by csl26-7hsx: `resolve_localized_type_variant`'s callers passed `None` for the section-level `type_variants` fallback tier, so English items whose type wasn't redefined in the style's `en` locale override (e.g. `gbt7714.7.1.3:2`, the Coffee-drinking periodical) skipped straight to the locale block's flat, delimiter-less template — not a substitution/anonymous-author gap as originally diagnosed here.

After the fix, oracle re-run (`node scripts/oracle.js ... gb-t-7714-2025-numeric`) shows raw bibliography matches rising from 143/203 to 146/203. Of the original 18 tracked ids: 4 now raw-match the oracle exactly (`gbt7714.7.2.1:7`, `gbt7714.7.2.3:7`, `gbt7714.8.11.3.2:5`, `gbt7714.8.9.2:4`); the remaining 14 (including `gbt7714.7.1.3:2`) now have every structural component (title/volume/issue/year/pages) matching — their only remaining divergence is the pre-existing, already-registered Latin-script punctuation convention (full-width vs GB/T's own Latin half-width rule; see csl26-5y6k / MULTILINGUAL.md §3.2a), not a new or distinct bug.

Remaining scope for this bean: re-triage the *other* raw bibliography failures (57 total, up from an original baseline of ~60) not among these 18 tracked ids, since this fix's scope was limited to the locale-type-variant fallback.

## Session plan (2026-07-22)

Reconciled with report-core.js adjusted numbers: raw 146/203, adjusted ~192/203
(fidelityScore 0.959). Remaining 11 adjusted failures split:
- 3 genuine bugs (idx45 Sagan AV/television, idx46 Bengio EB/OL, idx47 Brown v.
  Board legal case) — from default 47-ref set, missing gb-t type-variants.
- 8 era/EDTF divergence candidates, to be adjudicated against gb7714-bench
  expected outputs before registering div-011 or fixing as bugs.

- [x] ~~Fix type-variant gap~~ deprioritized: types don't exist in 203-item GB/T corpus
- [x] Adjudicated 8 era/EDTF items against the official GB/T 7714-2025 standard text (gb7714-bench itself has no gold strings; used data/GB-T_7714-2025.original.toml, extracted from the standard PDF, via the typst-doc-cn/bib-csl-dev-data repo it references). All 8 confirmed: Citum matches the standard's own worked examples (§7.5.4.1, §7.5.4.3, §8.2.2, §8.4.2, §8.12.3) exactly; citeproc-js diverges from the standard in every case.
- [x] Registered div-011 in oracle-divergences.js + verification-policy.yaml
- [x] None needed — all 8 were citeproc-js/CSL-M oracle bugs, not Citum bugs
- [x] Regenerate docs/compat.html and confirm GB/T rows + adjusted score appear — see "Summary of Changes": regenerated in full

## Finding: the 3 "type-variant" bugs are out-of-corpus

Checked tests/fixtures/test-items-library/gb-t-7714-2025.json (203 items): types
present are book, article-journal, graphic, report, standard, patent, webpage,
dataset, thesis, manuscript, article-newspaper, chapter, paper-conference,
software, personal_communication, map, article. **No broadcast, legal_case, or
interview items exist in the corpus at all.** The 3 mangled renders (Sagan/
Bengio/Brown) come from tests/fixtures/references-expanded.json (the generic
47-ref default set merged into report-core's 250-total), not the GB/T corpus.
gb7714-bench and native reviewers will never exercise these types through this
style. Deprioritized below the 203-corpus adjudication work; may become a
separate lower-priority robustness bean rather than blocking this pass.

## Numeric result: 100% adjusted fidelity on the 203-item GB/T corpus

benchmarkRunResults now: status=pass, fidelityScore 0.989 (merged 250-ref report;
203-corpus-scoped run is 100% adjusted). Verified via
`node scripts/report-core.js --style gb-t-7714-2025-numeric` after clearing
.oracle-cache/report-core (cache was stale — its key hashes oracle.js but not
oracle-divergences.js/verification-policy.yaml, so it silently served pre-fix
results; worth flagging as a cache-key gap, see reporting section).

## Summary of Changes

Reconciled the 146/203 raw vs "100% adjusted" discrepancy: docs/compat.html was
stale (no GB/T rows at all, pre-dating the embedded family). Adjudicated the 8
remaining era/EDTF bibliography mismatches against the official GB/T 7714-2025
standard's own worked examples (extracted PDF text in
typst-doc-cn/bib-csl-dev-data, referenced by the gb7714-bench project this
session was prompted by — gb7714-bench itself has no gold strings, it's a
cross-engine visual comparison). All 8 confirmed as citeproc-js/CSL-M oracle
bugs, not Citum bugs; registered as div-011 in oracle-divergences.js +
verification-policy.yaml, with unit test coverage in oracle.test.js.

Result: gb-t-7714-2025-numeric now hits 100% adjusted fidelity on the 203-item
corpus (benchmarkRunResults status: pass). gb-t-7714-2025-note reached the same
for free (shared base) — see csl26-ap2b, completed. gb-t-7714-2025-author-date
was root-caused but not fixed (best-effort scoping decision) — see csl26-6eak
for the full write-up and fix recipe.

Also fixed a stale-cache bug in report-core.js (benchmark cache key didn't
hash oracle-divergences.js/verification-policy.yaml) and regenerated
docs/compat.html in full. Full record:
docs/architecture/audits/2026-07-22_GBT_DATE_ANNOTATION_FIDELITY.md.

Remaining out-of-corpus gap (broadcast/interview/legal_case type-variants,
merged-report-only, not in the 203-item corpus) deliberately left unfixed —
doesn't affect the number gb7714-bench or native reviewers will see.
