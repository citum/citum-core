---
# csl26-gtat
title: apa container packaging follow-up
status: todo
type: task
priority: high
created_at: 2026-04-09T15:40:00Z
updated_at: 2026-04-09T15:40:00Z
---

Continue the APA rich bibliography closure pass by fixing the bounded
container-packaging cluster in `apa-test-library-diagnostic`.

Current verified state:
- baseline APA gate remains `40 / 40`
- supplemental APA diagnostic benchmark is `41 / 74`
- this bean owns rows `44`, `46`, `47`, `48`, `56`, and `59`
- target references: magazine articles, chapter-in-report rows, technical
  reports, and book/report packaging with edition / volume / translator data

Expected owning subsystem:
- primary: `citum_migrate` / schema-data conversion
- secondary: `citum_engine`
- APA style YAML only if the data is already present and the template is
  provably wrong

Current mismatch shape:
- translator / editor / edition / volume / report-number metadata is being
  flattened or dropped
- chapter-in-report rows collapse to `[Technical report]`-style fallbacks
- magazine packaging loses translator and special-format details

## Tasks
- [ ] Extract and save a reduced APA fixture for rows `44`, `46`, `47`, `48`,
  `56`, and `59`.
- [ ] Audit intake and conversion for translator, editor, edition, volume, and
  report-number preservation on this cluster.
- [ ] Restore APA container packaging for chapter-in-report and technical
  report rows.
- [ ] Restore issue / volume / date / title assembly for magazine rows.
- [ ] Re-run the reduced fixture and the full APA benchmark and record before /
  after counts in this bean.

## Acceptance
- rows `44`, `46`, `47`, `48`, `56`, and `59` match the oracle exactly
- report number, edition, volume, translator, and container title survive
  round-trip and render in the expected order
- baseline APA remains `40 / 40`

## Stop-Loss Rule
- Stop after 2 distinct implementation attempts with no net gain.
- Reclassify immediately as `style-defect`, `processor-defect`, or
  `migration-artifact` and hand off the unresolved rows explicitly.
