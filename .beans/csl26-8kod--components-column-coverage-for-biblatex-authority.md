---
# csl26-8kod
title: Components column coverage for biblatex-authority styles
status: todo
type: task
priority: normal
tags:
    - testing
    - migrate
created_at: 2026-03-07T19:25:01Z
updated_at: 2026-04-25T20:20:07Z
---

The five compound-numeric styles (numeric-comp, chem-acs, angewandte-chemie, chem-rsc, chem-biochem) use biblatex as their benchmark authority. The biblatex oracle in report-core.js builds entries as { expected, actual, match } with no component-level breakdown, so computeComponentMatchRate() always returns null and the Components column in compat.html is empty for these styles.

## Problem

The citeproc-js oracle runs a full component diff (parseComponents on expected vs actual), populating entry.components.matches and entry.components.differences. No equivalent exists for biblatex output.

## Candidate Solutions

**Option A — Post-hoc component diff on flat strings**
After the biblatex oracle produces { expected, actual } pairs, run the same parseComponents / oracle-utils.js component parser on both strings and diff the results. Risk: the component schema may not transfer cleanly to biblatex output format.

**Option B — Structured biblatex snapshots**
Change biblatex snapshot format from flat string arrays to per-field JSON (one object per entry with named fields). This enables field-level comparison but requires regenerating all snapshots and updating oracle infrastructure.

**Option C — Citum structured render diff**
Add a separate oracle pass for biblatex styles using the Citum JSON render output (citum render refs --json), which has structured fields. Diff these structured fields against a structured biblatex reference (e.g. from biber --tool output or hand-authored YAML). This is the cleanest long-term solution but requires new reference data.

**Option D — Bibliography pass-rate proxy (rejected)**
Fall back to treating entry.match as 1/1 per entry. Rejected: this duplicates the fidelity score and gives no additional signal about which components are passing or failing.
