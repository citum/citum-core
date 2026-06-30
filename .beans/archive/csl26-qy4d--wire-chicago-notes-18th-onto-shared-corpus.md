---
# csl26-qy4d
title: Add notes-18th bibliography surface for shared corpus
status: completed
type: task
priority: normal
created_at: 2026-06-30T15:07:36Z
updated_at: 2026-06-30T16:02:02Z
parent: csl26-40n4
---

Follow-up from csl26-8br0: notes-18th cannot use the shared Chicago corpus until it has a bibliography surface (bibliography.template is [] today) or citeproc-oracle supports a citation-only scope (scripts/lib/verification-policy.js asserts scope != citation). Once either lands, wire chicago-notes-18th onto the shared-corpus benchmark_run in scripts/report-data/verification-policy.yaml to match the other three Chicago variants.

## Todo
- [x] Decided: lift the citeproc-oracle citation-scope restriction (smaller, correct fix vs. inventing a notes-18th bibliography surface)
- [x] Implemented: removed the oracle.js throw + made bibliography matching scope-aware; dropped the verification-policy.js assertion; inverted the pinning tests
- [x] Wired chicago-notes-18th onto the shared-corpus benchmark_run (scope: citation)
- [x] Confirmed: all four Chicago variants report on the shared corpus (notes-18th citation 7/15; others citation+bibliography)

## Summary of Changes

Lifted the citeproc-oracle citation-only scope restriction rather than working around it (the original ground-prep commit deferred notes-18th here).

- scripts/oracle.js: removed the early throw for --scope citation; bibliography matching is now skipped for citation scope (the mirror of how scope: bibliography skips citations). Citation-only runs grade citations and report bibliography total 0.
- scripts/lib/verification-policy.js: dropped the assertion that blocked scope: citation for citeproc-oracle benchmark runs. The citations_fixture requirement for non-bibliography scopes is unchanged.
- scripts/report-data/verification-policy.yaml: chicago-notes-18th now carries a chicago-shared-corpus benchmark_run with scope: citation (count_toward_fidelity: false), so all four Chicago variants report on the shared corpus.
- Tests: inverted oracle.test.js and report-core.test.js cases that pinned the old throw/assertion into positive assertions that citation-only is supported. 84/84 script tests pass.

Note: giving chicago-notes-18th an actual bibliography surface remains separate substrate work (it has no bibliography by design); this bean only concerned getting it onto the shared corpus, which its citation surface now satisfies.
