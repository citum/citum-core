---
# csl26-mwnt
title: apa year-suffix disambiguation cleanup
status: todo
type: task
priority: normal
created_at: 2026-04-09T15:40:00Z
updated_at: 2026-04-09T15:40:00Z
---

Own any residual APA year-letter or anonymous-ordering mismatches that remain
after the structural rich-bibliography fixes land.

Current verified state:
- baseline APA gate remains `40 / 40`
- supplemental APA diagnostic benchmark is `41 / 74`
- this bean does not own current rows until structural clusters are re-run
- year-suffix cleanup must be last because many current year-letter deltas are
  downstream effects of lossy intake or fallback packaging

Expected owning subsystem:
- primary: `citum_engine`
- secondary: style YAML only if processor output is correct and APA suffix
  ordering is still wrong

## Tasks
- [ ] Wait until the web-native, container-packaging, and authored /
  containerized clusters have been re-run.
- [ ] Extract any rows where the only remaining difference is year suffix,
  anonymous ordering, or disambiguation ordering.
- [ ] Fix the residual disambiguation or anonymous-ordering behavior in one
  bounded processor pass.
- [ ] Re-run the reduced fixture and the full APA benchmark and record before /
  after counts in this bean.

## Acceptance
- no row remains mismatched solely because Citum assigns different year letters
  or anonymous ordering after the structural data is restored
- baseline APA remains `40 / 40`

## Stop-Loss Rule
- Do not edit year-suffix logic before the structural buckets have stabilized.
- Stop after 2 distinct processor attempts with no net gain and reclassify as
  intentional divergence only if the oracle behavior is non-portable or
  inconsistent.
