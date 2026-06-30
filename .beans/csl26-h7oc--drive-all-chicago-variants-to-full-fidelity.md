---
# csl26-h7oc
title: Drive all Chicago variants to full fidelity
status: todo
type: epic
priority: high
created_at: 2026-06-30T14:30:24Z
updated_at: 2026-06-30T18:45:58Z
parent: csl26-40n4
---

Coordinator for driving all four Chicago variants from their real baseline to
~100% fidelity + clean SQI, via the `style-tune` skill, one variant per child
bean. Supersedes the prior "final tuning pass" framing, which understated the
gap — baseline below was unchanged-codebase, freshly measured.

## Baseline (measured 2026-06-30, `node scripts/report-core.js`, `chicago-shared-corpus` run — 15 citations / 402 bibliography refs)

| variant | citations | bibliography | gated in CI? |
|---|---|---|---|
| chicago-author-date-18th | 11/15 (73%) | 298/402 (74%) | **bib only**, via separate `chicago-zotero-bibliography` run, `min_pass_rate: 0.73` (barely above floor) |
| chicago-notes-18th | 7/15 (47%) | — (no bibliography surface) | no |
| chicago-shortened-notes-bibliography | 6/15 (40%) | 264/402 (66%) | no |
| taylor-and-francis-chicago-author-date | 11/15 (73%) | 298/402 (74%, inherits author-date) | no |

Only one of these eight numbers is gated today. The `chicago-shared-corpus`
benchmark run exists identically across all four variants in
`scripts/report-data/verification-policy.yaml` but is `count_toward_fidelity:
false` (diagnostic-only). Every citation surface, and the bibliography surface
for notes/shortened/T&F, can regress silently.

## Recommendations — pending decision (not yet acted on)

**Gating policy:** promote `chicago-shared-corpus` to `count_toward_fidelity:
true` on all four variants with interim floors set just below current
baseline (lock against regression during tuning), then ratchet upward per
variant as fidelity climbs. Suggested initial floors: author-date ~0.70, T&F
~0.70, notes ~0.45 (citation-only), shortened ~0.40 (combined). Once
shared-corpus gates author-date's bibliography, demote the now-redundant
`chicago-zotero-bibliography` run (same `chicago-18th.json` fixture) to
diagnostic — retires the duplicate gate flagged on #987.

**Tuning order:** author-date → T&F → notes → shortened. T&F-core extends
`chicago-author-date-18th`, so author-date gains lift T&F before its Style-F
deltas apply. `chicago-shortened-notes-bibliography-core` extends
`chicago-notes-18th`, so notes is tuned before shortened inherits its citation
gains, leaving shortened to only need its own bibliography surface tuned.
Author-date/T&F (73-74%) are closest to the bar; notes (47%) and shortened
(40%) need the most work but inherit upstream wins.

## Todo
- [ ] Decide and implement the gating-policy recommendation above (own commit
      to `verification-policy.yaml`, separate from the four tunes)
- [ ] Land child tune: chicago-author-date-18th
- [ ] Land child tune: taylor-and-francis-chicago-author-date
- [ ] Land child tune: chicago-notes-18th
- [ ] Land child tune: chicago-shortened-notes-bibliography
- [ ] Final `report-core.js` sweep confirming all four at target; demote/retire
      redundant gating runs
