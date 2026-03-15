---
# csl26-ggtg
title: 'Rust-simplify: engine + migrate refactor'
status: completed
type: task
priority: normal
created_at: 2026-03-15T10:57:45Z
updated_at: 2026-03-15T11:14:59Z
---

Three-task rust-simplify pass: (1) citum-migrate/compilation.rs dead code + lookahead helper, (2) node_compiler.rs override-building helper, (3) citum-engine/grouped.rs split + GroupRenderParams struct. Via PR branch refactor/rust-simplify-engine-migrate.

## Summary of Changes

- compilation.rs: removed dead compile_with_wrap (140 lines) and add_or_upgrade_component (73 lines); extracted merge_text_lookahead helper
- node_compiler.rs: extracted build_type_overrides helper, deduplicating 3 call sites in compile_variable
- grouped.rs: introduced GroupRenderParams struct; updated render_fallback_grouped_citation_with_format and render_group_item_parts_with_format to use it, eliminating all too_many_arguments allow attributes
- grouped_fallback.rs: new submodule housing GroupRenderParams definition
- All 706 tests pass, clippy clean
