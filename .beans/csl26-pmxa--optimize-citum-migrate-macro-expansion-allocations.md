---
# csl26-pmxa
title: Optimize citum-migrate macro expansion allocations
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

Reduce peak heap and total allocation in `citum-migrate` macro expansion.

## Evidence

The profiling report at
`docs/architecture/2026-05-24_PERFORMANCE_PROFILE_REPORT.md` measured
`citum-migrate styles-legacy/apa.csl --template-source xml` at 142.0 ms and
333,974,675 bytes allocated. DHAT showed large peak allocations in
`MacroInliner::expand_macros_no_increment`, especially cloned CSL node subtrees
around `crates/citum-migrate/src/lib.rs:122`.

## Acceptance Criteria

- Macro expansion avoids cloning whole `CslNode` trees when only child vectors
  need to change.
- A focused benchmark, profiling fixture, or before/after DHAT comparison shows
  reduced peak heap on `styles-legacy/apa.csl`.
- Existing migration output fidelity stays unchanged for the measured APA path.
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
  and `cargo nextest run` pass.
