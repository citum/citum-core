---
# csl26-gc43
title: Fix GB/T 7714 numeric bibliography defects exposed by oracle harness fix
status: todo
type: bug
priority: normal
tags:
    - gb-t
    - fidelity
    - punctuation
    - rendering
created_at: 2026-07-24T13:19:22Z
updated_at: 2026-07-24T13:19:43Z
---

Fixing the oracle harness bugs tracked in csl26-7jib (normalizeText stripping
terminal punctuation before comparison; oracle-fast.js never applying the
STRICT_BIBLIOGRAPHY_STYLES exact-match gate) exposed real GB/T 7714 numeric
bibliography rendering defects that were previously invisible to
report-core.js's fidelity score. `node scripts/report-core.js --style
gb-t-7714-2025-numeric` now reports 238/250 bibliography matches (was
reporting 247/250 pre-fix -- the extra 9 were false passes masked by the
harness bugs, not real).

14 entries currently fail with genuine text differences (not structural
issues). Representative examples from `oracleDetail`:

- `[39]` missing bracketed access-date `[2019]` before the URL.
- `[45]` wrong medium code (`[CP/OL]` vs correct `[CP/Apparatus]`) and a
  missing period before the URL.
- `[22]` missing spaces and wrong bracket form around `[Z/film]` (renders
  `[Film]` with no surrounding spacing).
- `[44]`, `[24]` missing parenthetical publication dates entirely.
- `[74]`, `[75]`, `[78]`, `[98]`, `[167]`, `[169]` -- citum renders a GB/T
  era/date annotation (e.g. `（民国三十六年）`, `印刷`, `（清同治四年）`) that the
  citeproc-js oracle does not. This may be a legitimate div-011-class
  oracle/citum divergence (see `docs/architecture/audits/` and
  `scripts/lib/oracle-divergences.js`'s `GBT_DATE_ANNOTATION_PATTERNS`) rather
  than a citum bug -- needs adjudication against the GB/T 7714-2025 standard's
  own worked examples before deciding whether to fix the renderer or register
  a new oracle divergence.
- `[79]` bracket-form mismatch: oracle `1936.` vs citum `[1936].`.
- `[111]` citum renders an extra `[1990]` bracket the oracle doesn't.
- `[178]` citum duplicates the CSTR identifier after the URL (related to,
  possibly the same defect as, the already-tracked `csl26-ia43`).

Scope: investigate each, fix genuine Citum rendering bugs, and for the
date-annotation cluster specifically, determine whether it's a new div-011-
style oracle divergence or a real defect before touching the renderer.
