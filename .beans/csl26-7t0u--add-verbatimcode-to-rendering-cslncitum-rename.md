---
# csl26-7t0u
title: Add verbatim/code to Rendering + Citum→Citum rename
status: completed
type: task
created_at: 2026-03-04T12:44:25Z
updated_at: 2026-03-04T12:44:25Z
---

## Tasks

- [x] Add `verbatim: Option<bool>` and `code: Option<bool>` to `Rendering` struct in `citum_schema/src/template.rs`, update `merge()`, and propagate through all component structs
- [x] Apply verbatim/code in engine renderers (format.rs, html.rs, org.rs, plain.rs, djot.rs, latex.rs, typst.rs)
- [x] Bulk replace Citum→Citum in comments, docs, bean files, scripts (skip csl-legacy/ source)
- [x] Spot-check ~5 random files after rename to verify correctness
- [x] Run pre-commit checks and commit (no push)
