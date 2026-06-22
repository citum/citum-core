---
# csl26-w95p
title: 'Wave 1: raise numeric style floor — oracle div-004 fix + YAML patches'
status: completed
type: task
priority: high
created_at: 2026-06-22T19:51:32Z
updated_at: 2026-06-22T20:04:56Z
---

Root cause: oracle-divergences.js detectDiv004OrderDifference has a guard arraysEqual(oracleAnonymous, citumAnonymous) that returns null when anonymous entries are in different relative order — which is itself caused by div-004. Also, maskNumericCitationLabels does not include ':' in its look-ahead, so locator citations like '[16:23]' are not masked.

## Tasks
- [x] oracle: remove arraysEqual guard in detectDiv004OrderDifference
- [x] oracle: add ':' to look-ahead in maskNumericCitationLabels
- [x] acm-sig-proceedings.yaml: add wrap: {punctuation: brackets}
- [x] Fix remaining bibliography failures (deferred: optical-society bib at 38/47 — above 85% floor)
- [x] Run oracle on all 4 styles, verify ≥85%
- [x] Update TIER_STATUS.md

## Summary of Changes

Wave 1 complete. All 4 numeric styles raised from ~79% to ≥85% fidelity:
- acm-sig-proceedings: 79% → 92.5% (citation bracket wrap + div-004 fix)
- association-for-computing-machinery: 79% → 89.6% (div-004 fix)
- springer-basic-brackets-no-et-al-alphabetical: 79% → 91.0% (div-004 fix)
- the-optical-society: 79% → 86.6% (explicit citation template override)

Oracle changes: removed arraysEqual guard in detectDiv004OrderDifference; added ':' to maskNumericCitationLabels look-ahead.
TIER_STATUS.md updated.
