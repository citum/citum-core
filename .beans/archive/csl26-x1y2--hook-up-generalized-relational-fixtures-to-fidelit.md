---
# csl26-x1y2
title: Hook up generalized relational fixtures to fidelity reporting
status: todo
type: task
priority: normal
created_at: 2026-04-01T15:00:00Z
updated_at: 2026-04-02T00:23:32Z
---

Following the implementation of the generalized relational container model (PR #485 / v0.20.0) and the numbering cleanup, the fidelity pipeline now needs richer benchmark inputs that exercise the new relational model instead of relying only on the older flat fixtures.

This work should focus on building that broader capability first. Native relational corpora such as `examples/comprehensive.yaml` and `examples/chicago-bib.yaml` should become benchmarkable inputs, and external corpora such as the Zotero test libraries should plug into the same framework where they expose larger real-world style gaps.

This bean is specified by `docs/specs/FIDELITY_RICH_INPUTS.md`.

## Context
The architectural shift to recursive `WorkRelation` and canonical `numbering` was intended to prepare the ground for richer fidelity data that can drive later style enhancement waves. Small flat fixtures underrepresent container hierarchies, numbering semantics, reviewed/original relations, and large bibliography corpora.

## Tasks
- [ ] Add style-level `benchmark_runs` support to the fidelity reporting pipeline.
- [ ] Add native relational benchmark inputs for `examples/comprehensive.yaml` and `examples/chicago-bib.yaml`.
- [ ] Convert a small Zotero subset (e.g., the note-heavy Chicago 18 rows) into `examples/chicago-note-converted.yaml` using `scripts/export-chicago-note-examples.js` and include it as a native diagnostic fixture.
- [ ] Add an external bibliography pilot using Zotero test data from `tests/fixtures/test-items-library/`.
- [ ] Make richer relational inputs visible in report output so they can guide future style enhancement work.
