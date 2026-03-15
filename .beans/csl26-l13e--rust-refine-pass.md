---
# csl26-l13e
title: Rust refine pass
status: in-progress
type: task
priority: normal
created_at: 2026-03-15T13:16:42Z
updated_at: 2026-03-15T13:27:28Z
---

Reviewer-pattern refinement pass. One file per session.

## 2026-03-15 frontier
- This bean is the rolling refine log for `csl26-7p9u`, `csl26-ey6s`, and
  `csl26-5zzb`.
- Each child bean should append a dated note here after its `rust-refine` pass
  lands.

## 2026-03-15 refine
- citum-cli/src/main.rs: introduce RenderContext<'a>, remove 5 #[allow(clippy::too_many_arguments)] suppressions (render_refs_human, render_refs_json, print_human_safe, print_human, print_json_with_format)

- citum-migrate/src/template_compiler/types.rs: collect_types_recursive → associated fn, remove #[allow(clippy::only_used_in_recursion)]

- citum-migrate/src/fixups/mod.rs: refine the public fixups surface into a
  documented facade over `media`, `locator`, and `template` submodules after
  the simplify split, keeping `main.rs` call sites stable
