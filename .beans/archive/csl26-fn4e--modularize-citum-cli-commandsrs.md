---
# csl26-fn4e
title: Modularize citum-cli commands.rs
status: completed
type: task
priority: normal
created_at: 2026-05-16T23:15:04Z
updated_at: 2026-05-28T23:53:22Z
parent: csl26-rfct
---

Split crates/citum-cli/src/commands.rs (2923 lines) into a commands/ submodule hierarchy.

Closes item #3 on parent bean csl26-rfct.

## Scope (Modularize + small DRY wins)

- [x] Create commands/ submodule hierarchy
  - [x] commands/mod.rs — run() dispatcher, From<RefsFormat>, CliResult alias, re-exports
  - [x] commands/catalog.rs — StyleCatalogRow, CatalogSourceFilter, page/print helpers
  - [x] commands/registry.rs — registry types, I/O helpers, run_registry_*
  - [x] commands/style.rs — style query commands + helpers
  - [x] commands/style_install.rs — run_style_add and run_style_uninstall + CliStyleBrowserActions
  - [x] commands/locale.rs — locale commands
  - [x] commands/doctor.rs — DoctorReport + run_doctor
  - [x] commands/lint.rs — lint commands
  - [x] commands/check.rs — run_check + check_style_input
  - [x] commands/convert.rs — run_convert_*
  - [x] commands/render/{mod,human,json}.rs
  - [x] commands/schema.rs (schema feature)
  - [x] commands/bindings.rs (typescript feature)
  - [x] commands/util.rs — confirm + validate_resource_name
  - [x] serialize_any/deserialize_any inlined in commands/convert.rs (only caller)
- [x] DRY: introduce CliResult<T = ()> type alias
- [x] DRY: dedupe style_catalog_row / installed_style_catalog_row
- [x] DRY: centralize confirm() helper
- [x] Move tests into per-module cfg(test) blocks
- [x] fmt and clippy pass under --workspace
- [x] cargo nextest run --workspace passes (1298/1298)
- [x] Smoke: --help, style list, doctor all work
- [x] Regenerate docs/schemas/ (no diff)
- [x] Check off item #3 on parent csl26-rfct
- [x] Open PR, watch CI to green (went direct to main — no PR needed)


## Summary of Changes

- Split `crates/citum-cli/src/commands.rs` (2923 lines) into a `commands/` submodule hierarchy: 15 sibling files plus a `render/` subdir (3 files).
- Added per-family `dispatch(command)` functions in each module so the top-level `run()` collapsed from 95 lines to a 23-line match.
- Added `pub(super) type CliResult<T = ()> = Result<T, Box<dyn Error>>;` alias in `commands/mod.rs`, replacing ~80 verbose signatures.
- Deduped `style_catalog_row` and `installed_style_catalog_row` into `StyleCatalogRow::from_entry` and `StyleCatalogRow::installed` associated functions.
- Centralized `confirm()` and `validate_resource_name()` into `commands/util.rs` (callers in 4 modules).
- Tests co-located with the code they exercise (per-module `#[cfg(test)] mod tests` blocks).
- Net delta: 17 files changed, +3220 / -2923 (≈+297 lines from module headers + dispatch shims); zero behavior change.
- 1298/1298 workspace tests pass; clippy clean under `--workspace --all-targets`.
- `docs/schemas/` unchanged (regenerated and identical).

## Summary of Changes (addendum)

Work landed in commit `5c61049e refactor(cli): split commands.rs into modules` on main. The `commands/` subdir exists with 15+ sibling files; `commands.rs` is gone. The earlier summary above covers the full scope.
