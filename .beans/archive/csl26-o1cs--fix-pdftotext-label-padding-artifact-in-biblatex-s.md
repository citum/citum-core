---
# csl26-o1cs
title: Fix pdftotext label-padding artifact in biblatex snapshots
status: completed
type: bug
priority: high
created_at: 2026-03-07T19:38:44Z
updated_at: 2026-03-07T19:44:55Z
---

The gen-biblatex-snapshot.js pipeline converts PDFs via pdftotext -layout, which emits extra spaces inside fixed-width label boxes. This produces entries like '(1 )' instead of '(1)' in snapshots for styles that use parenthetical numeric labels (e.g. chem-biochem).

Root cause: biblatex typsets citation labels in fixed-width boxes for hanging-indent alignment. pdftotext -layout faithfully captures the padding space.

Fix: add a cleanup step in extractBibliography() to strip intra-label padding:
  .map((e) => e.replace(/([(\[])\s*(\d+)\s*([)\]])/g, '$1$2$3'))

Then regenerate chem-biochem snapshot with --force.

## Summary of Changes

- Added label-padding cleanup step to `extractBibliography()` in `gen-biblatex-snapshot.js` (strips `(1 )` → `(1)` pdftotext artifact)
- Manually patched `tests/snapshots/biblatex/chem-biochem.json` — 25 entries cleaned (biblatex-chem not installable on dev machine, so script was applied in-place)
