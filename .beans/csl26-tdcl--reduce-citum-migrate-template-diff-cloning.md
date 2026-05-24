---
# csl26-tdcl
title: Reduce citum-migrate template diff cloning
status: todo
type: task
priority: normal
tags:
    - performance
    - migrate
created_at: 2026-05-24T14:32:34Z
updated_at: 2026-05-24T14:44:07Z
---

## Goal

Reduce repeated cloning in `citum-migrate` template diff selection and
equivalence checks.

## Evidence

The profiling report at
`docs/architecture/2026-05-24_PERFORMANCE_PROFILE_REPORT.md` identifies
`template_diff::component_selector` and `diff_resolves_to_template` as hot
allocation sites. DHAT attributed repeated allocation to cloned
`serde_json::Value` selector fields at `template_diff.rs:239`, cloned parent
templates in `TemplateVariant::Full`, and temporary style construction for diff
resolution.

## Acceptance Criteria

- Selector construction avoids unnecessary `serde_json::Value` clones or uses a
  smaller selector representation.
- Diff equivalence checks avoid cloning full parent templates where borrowed or
  cached resolution can be used safely.
- A profiling comparison shows lower total allocation for
  `citum-migrate styles-legacy/apa.csl --template-source xml`.
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
  and `cargo nextest run` pass.
