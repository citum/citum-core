---
# csl26-7jib
title: Investigate CSL-M oracle fixture staleness for gb-t-7714-2025-numeric
status: completed
type: task
priority: normal
tags:
    - oracle
    - fidelity
    - punctuation
    - testing
created_at: 2026-07-23T20:54:52Z
updated_at: 2026-07-24T13:19:59Z
---

tests/fixtures/csl-m/gb-t-7714-2025-numeric.csl (our local CSL-M oracle source, used by oracle.js/report-core.js) omits the terminal period on bare bibliography entries (e.g. the Hawking ITEM-2 fixture, ending in a bare year with no url/cstr/doi), matching citum's own (buggy) no-period output and therefore scoring it as passing. Real Zotero/citeproc-js output in the gb7714-bench CI artifact, rendered from upstream zotero-chinese/styles' current CSL, DOES have the period -- meaning our local CSL-M copy differs from upstream. This is why report-core.js's ~99% fidelity score for this style did not catch the missing-period defect tracked in csl26-iqxu. Investigate scope of the staleness (just this one macro, or broader drift) and refresh the fixture from upstream zotero-chinese/styles. See docs/architecture/audits/2026-07-23_GB7714_BENCH_COMPARISON.md 'Why our own fidelity number is blind to this' section for full context. Not a case for registering an oracle-divergences.js entry (unlike div-011) -- here the oracle and the standard agree; only our local oracle copy and citum's rendering are wrong.

## Corrected finding

The staleness theory was wrong. `tests/fixtures/csl-m/gb-t-7714-2025-numeric.csl`
is byte-identical to `zotero-chinese/styles` upstream `main` HEAD (verified via
`gh api repos/zotero-chinese/styles/contents/...` and `commits?path=...` -- no
commit has touched that style's source directory since our SOURCE.md pin,
`363713c6...`, 2026-07-05). There was nothing to refresh.

The real mechanism was two harness bugs in `scripts/oracle-utils.js` /
`scripts/oracle-fast.js`:

1. `normalizeText` unconditionally stripped trailing `.,;:` from both oracle and
   citum text before every comparison and before display -- masking any
   terminal-punctuation-only defect, for every style, not just GB/T.
2. `scripts/oracle-fast.js` (the snapshot-based path `report-core.js` actually
   calls for GB/T's headline score) never applied `oracle.js`'s
   `bibliographyComparisonMatches`/`STRICT_BIBLIOGRAPHY_STYLES` exact-match
   gate -- it always fell back to a lenient 0.60 token-similarity threshold,
   which is blind to punctuation entirely. This was actively hiding 8/47 (17%)
   real GB/T 7714 numeric bibliography defects from the fidelity score at
   investigation time, independent of bug 1.

Both fixed harness-wide (per user direction: global scope, not GB/T-only).
Full-corpus `report-core.js` diff (all 155 styles, before/after) confirms only
the 3 GB/T 7714 styles' pass counts moved; no other style regressed. Two
non-GB/T comparator edge cases surfaced during that diff and were fixed
alongside (div-005 archive-fragment stripping assumed no trailing punctuation;
`compareText`'s case-mismatch check could be fooled by a co-occurring
punctuation difference) -- both restored to their pre-change baseline counts
after the fix.

GB/T 7714 numeric's real bibliography fidelity is now 238/250 (was reporting
247/250), exposing genuine Citum rendering defects tracked in a follow-up bean.

See PR for full diff and verification detail.

## Summary of Changes

Investigation overturned the bean's premise (fixture staleness) and found two
real harness bugs instead -- see the corrected finding above for the full
mechanism. Fixed both, harness-wide, per user direction (global scope):

- `scripts/oracle-utils.js`: `normalizeText` no longer strips trailing
  `.,;:` before comparison/display. Also hardened `compareText`'s
  case-mismatch detection to stay correct when a trailing-punctuation
  difference co-occurs with a real case difference (needed after removing
  the blanket strip -- caught by a full-corpus before/after diff, not by
  the pre-existing unit tests).
- `scripts/oracle.js`: exported `STRICT_BIBLIOGRAPHY_STYLES` so it has one
  source of truth.
- `scripts/oracle-fast.js`: now imports and applies
  `bibliographyComparisonMatches` (oracle.js's strict exact-match gate for
  GB/T 7714 styles) in its bibliography pairing loop, instead of the lenient
  similarity-threshold fallback it silently used before.
- `scripts/lib/oracle-divergences.js`: `stripTrailingArchiveFragments`
  (div-005) now tolerates a trailing terminal mark after the archive
  fragment, since citation text can legitimately end in one now that it
  isn't stripped upstream.
- `scripts/report-migrate-sqi.js`: `normalizedEqual` keeps its own local
  trailing-punctuation tolerance (a different, legitimate use case --
  comparing oracle snapshots to each other for fixture-minimization
  acceptance, not citum's rendering against the oracle).

Verified via a full 155-style `report-core.js` before/after diff: only the 3
GB/T 7714 styles' pass counts moved (238/250, 28/48, 238/250 bibliography vs.
their previously-inflated 247/250, 41/48, 247/250); zero other styles
regressed. Two non-GB/T comparator edge cases (Chicago Notes div-005 citation,
MLA case-mismatch detection) surfaced during that diff, were root-caused and
fixed above, and are back to their exact pre-change baseline counts.
`node --test scripts/*.test.js scripts/lib/*.test.js` -- 196/196 pass,
including new regression tests for both fixes and the two edge cases.

Filed csl26-gc43 for the 14 real GB/T 7714 numeric bibliography defects this
fix exposed (out of scope here -- this bean was about the harness, not the
renderer).
