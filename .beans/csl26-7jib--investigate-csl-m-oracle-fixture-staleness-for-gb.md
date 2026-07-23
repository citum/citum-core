---
# csl26-7jib
title: Investigate CSL-M oracle fixture staleness for gb-t-7714-2025-numeric
status: todo
type: task
priority: normal
created_at: 2026-07-23T20:54:52Z
updated_at: 2026-07-23T20:55:11Z
---

tests/fixtures/csl-m/gb-t-7714-2025-numeric.csl (our local CSL-M oracle source, used by oracle.js/report-core.js) omits the terminal period on bare bibliography entries (e.g. the Hawking ITEM-2 fixture, ending in a bare year with no url/cstr/doi), matching citum's own (buggy) no-period output and therefore scoring it as passing. Real Zotero/citeproc-js output in the gb7714-bench CI artifact, rendered from upstream zotero-chinese/styles' current CSL, DOES have the period -- meaning our local CSL-M copy differs from upstream. This is why report-core.js's ~99% fidelity score for this style did not catch the missing-period defect tracked in csl26-iqxu. Investigate scope of the staleness (just this one macro, or broader drift) and refresh the fixture from upstream zotero-chinese/styles. See docs/architecture/audits/2026-07-23_GB7714_BENCH_COMPARISON.md 'Why our own fidelity number is blind to this' section for full context. Not a case for registering an oracle-divergences.js entry (unlike div-011) -- here the oracle and the standard agree; only our local oracle copy and citum's rendering are wrong.
