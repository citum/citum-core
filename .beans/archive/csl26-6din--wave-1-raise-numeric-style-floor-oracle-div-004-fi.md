---
# csl26-6din
title: 'Wave 1: raise numeric style floor — oracle div-004 fix + YAML patches'
status: scrapped
type: task
priority: high
created_at: 2026-06-22T19:51:21Z
updated_at: 2026-06-22T20:06:27Z
---

8 styles below 80%. Wave 1 targets 4 numeric styles (acm-sig-proceedings, association-for-computing-machinery, springer-basic-brackets-no-et-al-alphabetical, the-optical-society) stuck at ~79% due to div-004 oracle adjustment not firing.

Root cause: oracle-divergences.js detectDiv004OrderDifference has a guard arraysEqual(oracleAnonymous, citumAnonymous) that returns null when anonymous entries are in different relative order — which is itself caused by div-004 (Citum sorts anonymous by title). Also, maskNumericCitationLabels pattern doesn't include ':' in its look-ahead, so locator citations like '[16:23]' are not masked.

Fixes:
- [ ] oracle: remove arraysEqual(oracleAnonymous, citumAnonymous) guard in detectDiv004OrderDifference
- [ ] oracle: add ':' to look-ahead in maskNumericCitationLabels pattern
- [ ] acm-sig-proceedings.yaml: add wrap: {punctuation: brackets} to citation config
- [ ] Fix remaining bibliography failures in affected styles
- [ ] Run oracle on all 4 styles to verify ≥85% fidelity
- [ ] Update docs/TIER_STATUS.md

## Reasons for Scrapping\n\nDuplicate of csl26-w95p — same wave, different ID created in error.
