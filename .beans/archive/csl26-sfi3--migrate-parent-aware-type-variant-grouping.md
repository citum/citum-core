---
# csl26-sfi3
title: 'migrate: parent-aware type-variant grouping'
status: completed
type: bug
priority: normal
created_at: 2026-06-15T22:21:14Z
updated_at: 2026-06-15T22:30:00Z
parent: csl26-vmcr
---

Follow-up split from csl26-e94m. The attempted normalization that grouped
identical bibliography type-variant templates is not safe yet.

## Diagnosis

The regression is not that the engine cannot match `TypeSelector::Multiple`.
The bibliography path does match multi-selectors.

The actual bug is migration ordering. The experimental grouping pass grouped
raw type templates before lineage/wrapper diffing and before wrapper semantics
selected the final parent-aware template. In `china-information`, raw `book`
equals `bill`, so grouping created a shared selector for `bill,book,...`. After
wrapper resolution, however, `book` needs a larger parent-aware template. The
early multi-selector shadows that resolved template and regresses output.

`engine_validate_variants` is still useful as a structural safety net for bad
diffs, but it cannot prove that early grouping selected the semantically right
parent/template before wrapper resolution.

## Scope

- Do not reintroduce raw-template grouping in `template_diff.rs`.
- Design a parent-aware grouping pass that runs only after final resolved
  template and wrapper semantics are known.
- Alternatively, disable grouping for wrapper output until a parent-aware pass
  exists.
- Preserve original sibling-mining behavior until grouping is made
  parent-aware.
- Keep temporary debug controls out of the production path; this should not
  require `CITUM_DBG_*` or `CITUM_DISABLE_VARIANT_GROUPING` hooks.

## Acceptance

- `styles-legacy/china-information.csl` does not regress when grouping is
  enabled.
- Grouped selectors preserve distinct resolved output for `book` versus `bill`
  when wrapper semantics require different templates.
- Seeded random-100 SQI shows no fidelity regressions against the current
  csl26-e94m baseline.
- Full repo gate passes before shipping any implementation.

## References

- PR #929 RTM decision: ship csl26-e94m and defer csl26-sfi3.
- Migration surface: `crates/citum-migrate/src/template_diff.rs`.
- Engine resolver surface:
  `crates/citum-schema-style/src/template/resolution.rs`.

## Closure

Completed by locking the migration-order invariant in `template_diff.rs`:
pre-wrapper type-variant construction preserves original selector keys and does
not synthesize `TypeSelector::Multiple` from raw-equal templates. Sibling diff
mining remains allowed because it keeps selectors addressable. Parent-aware
post-resolution grouping remains deferred until it can run after final
child/parent wrapper templates are known.
