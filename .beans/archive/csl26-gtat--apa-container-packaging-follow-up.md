---
# csl26-gtat
title: apa container packaging follow-up
status: completed
type: task
priority: high
created_at: 2026-04-09T15:40:00Z
updated_at: 2026-04-10T22:09:13Z
---

Continue the APA rich bibliography closure pass by fixing the bounded
container-packaging cluster in `apa-test-library-diagnostic`.

Current verified state:
- APA citations remain `40 / 40`
- APA bibliography is now `54 / 54`
- the reduced `chapter,report,article-magazine,book` cluster is `17 / 17`
- the historical `41 / 74` note and six-row ownership list were stale on this
  branch before implementation
- the live branch truth narrowed to a single residual mismatch,
  `6188419/4JYXEPMY`, before the final fix landed

Expected owning subsystem:
- primary: `citum_migrate` / schema-data conversion
- secondary: `citum_engine`
- APA style YAML only if the data is already present and the template is
  provably wrong

Implemented fix:
- preserve parent volume as canonical collection numbering for collection
  components
- avoid empty container-editor groups when a chapter has no named parent
- preserve a component-level editor fallback for no-parent chapters so
  role-substitute paths still resolve cleanly
- restructure the APA chapter `In` group so it only renders when the group has
  real content, while keeping sentence-case `In` where it does render

Verification summary:
- `cargo fmt`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run` (`1012 / 1012` passed)
- `./scripts/check-docs-beans-hygiene.sh`
- `node scripts/extract-rich-benchmark-cluster.js --style apa-7th --benchmark apa-test-library-diagnostic --type chapter,report,article-magazine,book --only-mismatches --out-dir /tmp/apa-csl26-gtat-post-editor-fix`
- `node scripts/report-core.js --style apa-7th` with an extended one-off timeout shim to avoid the default oracle timeout
- `cargo run --bin citum --features schema -- schema --out-dir docs/schemas`

## Tasks
- [x] Refresh the reduced APA cluster against current branch truth.
- [x] Audit conversion and rendering for translator, editor, edition, volume,
  and report-number preservation.
- [x] Close the remaining chapter/report/book packaging gap in the live cluster.
- [x] Re-run the reduced fixture and full APA report and record the final
  counts in this bean.

## Acceptance
- live reduced cluster is `17 / 17` with `unresolvedMismatchCount: 0`
- report number, edition, volume, translator, and container/editor packaging
  survive conversion and render without the previous empty-group regression
- APA citations remain `40 / 40`
- APA bibliography remains `54 / 54`

## Stop-Loss Rule
- Stop after 2 distinct implementation attempts with no net gain.
- Reclassify immediately as `style-defect`, `processor-defect`, or
  `migration-artifact` and hand off the unresolved rows explicitly.
