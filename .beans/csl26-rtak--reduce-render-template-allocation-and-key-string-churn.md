---
# csl26-rtak
title: Reduce render template allocation and key-string churn
status: todo
type: task
priority: normal
tags:
    - engine
    - performance
created_at: 2026-05-24T14:32:34Z
updated_at: 2026-05-24T14:44:07Z
---

## Goal

Reduce allocation in large bibliography rendering without changing rendered
output.

## Evidence

The profiling report at
`docs/architecture/2026-05-24_PERFORMANCE_PROFILE_REPORT.md` measured large APA
rendering at 46.2 ms and 76,253,287 bytes allocated. DHAT and flamegraph data
pointed at `render_template_components` collection, `ProcEntry` cloning, cloned
citation/bibliography options, and high-frequency small allocations in
`key_base` / `get_variable_key`.

## Acceptance Criteria

- Rendering avoids per-template-component `RenderOptions` clones where a
  lightweight overlay or borrowed context is sufficient.
- Rendered component vectors are pre-sized, streamed, or otherwise allocated
  less aggressively.
- Key-base handling avoids avoidable `String` allocation in hot paths.
- A large-reference render comparison shows lower total allocation while keeping
  output unchanged.
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
  and `cargo nextest run` pass.
