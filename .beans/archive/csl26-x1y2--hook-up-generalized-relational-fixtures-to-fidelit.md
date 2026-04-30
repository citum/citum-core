---
# csl26-x1y2
title: Hook up generalized relational fixtures to fidelity reporting
status: completed
type: task
priority: normal
tags:
    - testing
created_at: 2026-04-01T15:00:00Z
updated_at: 2026-04-30T20:26:34Z
---

Following the implementation of the generalized relational container model (PR #485 / v0.20.0) and the numbering cleanup, the fidelity pipeline now needs richer benchmark inputs that exercise the new relational model instead of relying only on the older flat fixtures.

This work should focus on building that broader capability first. Native relational corpora such as `examples/comprehensive.yaml` and `examples/chicago-bib.yaml` should become benchmarkable inputs, and external corpora such as the Zotero test libraries should plug into the same framework where they expose larger real-world style gaps.

This bean is specified by `docs/specs/FIDELITY_RICH_INPUTS.md`.

## Context
The architectural shift to recursive `WorkRelation` and canonical `numbering` was intended to prepare the ground for richer fidelity data that can drive later style enhancement waves. Small flat fixtures underrepresent container hierarchies, numbering semantics, reviewed/original relations, and large bibliography corpora.

## Tasks
- [x] Add style-level `benchmark_runs` support to the fidelity reporting pipeline.
- [x] Add native relational benchmark inputs for `examples/comprehensive.yaml` and `examples/chicago-bib.yaml`.
- [x] Convert a small Zotero subset (e.g., the note-heavy Chicago 18 rows) into `examples/chicago-note-converted.yaml` using `scripts/export-chicago-note-examples.js` and include it as a native diagnostic fixture.
- [x] Add an external bibliography pilot using Zotero test data from `tests/fixtures/test-items-library/`.
- [x] Make richer relational inputs visible in report output so they can guide future style enhancement work.

## Summary of Changes

All tasks were completed as part of the rich-input pipeline work (merged ~2026-04-11, commit `8399cc2f` and surrounding commits):

- `report-core.js`: full `benchmark_runs` engine — `runCiteprocBenchmarkOracle`, `runNativeSmokeBenchmark`, `executeBenchmarkRuns`, tri-state status (pass/fail/ok), HTML display
- `scripts/report-data/verification-policy.yaml`: chicago-author-date wired with `chicago-zotero-bibliography` (citeproc-oracle, min_pass_rate: 0.73), `relational-comprehensive-smoke` and `relational-chicago-bib-smoke` (native-smoke); APA and AMA also have Zotero bibliography runs
- `examples/chicago-note-converted.yaml` present (7.6K); `examples/comprehensive.yaml` and `examples/chicago-bib.yaml` both configured as native-smoke inputs
- `tests/fixtures/test-items-library/` chicago-18th.json, apa-7th.json, ama-11th.json present and used

Bean was archived prematurely without being marked completed.
