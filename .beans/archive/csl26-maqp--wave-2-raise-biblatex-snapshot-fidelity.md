---
# csl26-maqp
title: 'Wave 2: raise biblatex snapshot fidelity'
status: completed
type: task
priority: high
created_at: 2026-06-22T20:19:46Z
updated_at: 2026-06-22T20:19:52Z
---

Regenerate biblatex snapshots for chem-rsc and numeric-comp to cover all 47 reference entries. Result: chem-rsc 78.7%→93.3%, numeric-comp 68.1%→93.6%.

## Summary of Changes

- Regenerated tests/snapshots/biblatex/chem-rsc.json (25→39 entries) via gen-biblatex-snapshot.js
- Regenerated tests/snapshots/biblatex/numeric-comp.json (33→47 entries) via gen-biblatex-snapshot.js
- Both snapshot expansions cover TLIB-SEL-* entries (types: manuscript, legislation, map, standard, bill, hearing, software, treaty, regulation) added to references-expanded.json after the original snapshots were generated.
- chem-rsc: 78.7% → 93.3% (42/47 bib)
- numeric-comp: 68.1% → 93.6% (44/47 bib)
