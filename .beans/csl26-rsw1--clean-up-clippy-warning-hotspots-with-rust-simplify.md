---
# csl26-rsw1
title: Clean up clippy warning hotspots with rust-simplify
status: completed
type: task
priority: normal
created_at: 2026-03-13T22:44:40Z
updated_at: 2026-03-14T15:15:00Z
---

Follow-up to archived bean `csl26-xtt0` and PR
[#362](https://github.com/citum/citum-core/pull/362).

Use the same `rust-simplify` approach to reduce the warning-heavy hotspots
still reported by Clippy, especially `too_many_lines` and
`cognitive_complexity`.

## Objective

Drive down the current warning set by simplifying responsibility boundaries and
extracting coherent helpers or private modules, not by silencing lints or
adding `#[allow(...)]`.

## Primary Targets

Start with the highest-value engine hotspots that remain after the processor
facade split:

- `crates/citum-engine/src/processor/document/mod.rs`
  - `process_document`
  - `prepare_note_citation_state`
- `crates/citum-engine/src/processor/rendering.rs`
  - `render_grouped_citation_with_format`
  - `process_template_request_with_format`
- `crates/citum-engine/src/values/contributor.rs`
  - `values`
  - `format_names`
  - `format_single_name`
- `crates/citum-engine/src/values/date.rs`
  - `values`
- `crates/citum-engine/src/values/number.rs`
  - `values`
- `crates/citum-engine/src/values/title.rs`
  - `values`
- `crates/citum-engine/src/values/variable.rs`
  - `values`
- `crates/citum-engine/src/io.rs`
  - `load_bibliography_with_sets`

Secondary cleanup targets once the engine warnings are materially reduced:

- `crates/citum-schema-style/src/locale/mod.rs`
- `crates/citum-schema-style/src/presets.rs`
- `crates/citum-schema-style/src/reference/conversion.rs`
- `crates/citum-migrate/src/analysis/citation.rs`
- `crates/citum-migrate/src/passes/deduplicate.rs`
- `crates/citum-migrate/src/passes/grouping.rs`
- `crates/citum-migrate/src/template_compiler/compilation.rs`
- `crates/citum-migrate/src/template_compiler/node_compiler.rs`
- `crates/citum-migrate/src/upsampler.rs`

## Checklist

- [x] Simplify `crates/citum-engine/src/processor/document/mod.rs`
  - [x] reduce `process_document`
  - [x] reduce `prepare_note_citation_state`
- [x] Simplify `crates/citum-engine/src/processor/rendering.rs`
  - [x] reduce `render_grouped_citation_with_format`
  - [x] reduce `process_template_request_with_format`
- [x] Simplify `crates/citum-engine/src/values/contributor.rs`
  - [x] reduce `values`
  - [x] reduce `format_names`
  - [x] reduce `format_single_name`
- [x] Simplify `crates/citum-engine/src/values/date.rs`
  - [x] reduce `values`
- [x] Simplify `crates/citum-engine/src/values/number.rs`
  - [x] reduce `values`
- [x] Simplify `crates/citum-engine/src/values/title.rs`
  - [x] reduce `values`
- [x] Simplify `crates/citum-engine/src/values/variable.rs`
  - [x] reduce `values`
- [x] Simplify `crates/citum-engine/src/io.rs`
  - [x] reduce `load_bibliography_with_sets`
- [x] Re-run `cargo clippy --all-targets --all-features -- -D warnings` once the
      current target slice is complete
- [x] Archive or update this bean when the warning cleanup frontier changes

## Rust-Simplify Direction

- Work one warning-heavy file per session unless a split naturally spans one
  tightly coupled sibling module.
- Prefer extracting private helpers first; if a file is still carrying multiple
  responsibilities, split by subsystem into private modules.
- Preserve public APIs and output behavior unless a separate change explicitly
  broadens scope.
- Add or update focused regression tests when a simplification changes control
  flow or module boundaries.
- Prefer clearer ownership over merely making functions shorter.
- Do not use lint suppression as the main fix.

## Acceptance Criteria

- Each touched hotspot is materially easier to read, not just line-wrapped.
- `cargo fmt --all --manifest-path Cargo.toml` passes.
- `cargo test -p citum-engine` passes for engine-focused slices.
- For broader slices, run `cargo clippy --all-targets --all-features -- -D warnings`
  after the targeted `rust-simplify` pass is complete.
- Record the before/after simplification result in the bean notes or commit
  message, especially when a file remains above the lint threshold for good
  reasons.

## Notes

- The processor facade work in PR #362 intentionally left `document/mod.rs`,
  `rendering.rs`, and the `values/*` modules as the next meaningful simplify
  frontier.
- `processor/tests.rs` may continue migrating toward the crate's BDD naming
  style, but only for behavior-contract tests; low-level unit coverage does not
  need to be renamed wholesale.

## Progress

- 2026-03-14: completed the first processor-focused slice by extracting
  document bibliography orchestration, note-state preparation helpers, grouped
  citation assembly helpers, and template-render component helpers; the
  remaining frontier is `values/*`, `io.rs`, and secondary non-engine targets.
- 2026-03-14: completed the values/* simplification by extracting
  `resolve_rendering_overrides`, `resolve_contributor_overrides`,
  `format_role_term`, and io parse helpers; reduced cognitive complexity
  and line count without lint suppressions. Remaining tasks: `values/title.rs`,
  `format_names`, `format_single_name`.
- 2026-03-14: extracted multilingual title config helper from title.rs::values,
  consolidating three consecutive `.and_then` calls into a single reusable
  `resolve_multilingual_title_config` function. Remaining: `format_names`,
  `format_single_name`.
- 2026-03-14: completed `format_names` and `format_single_name` refactoring by extracting
  `NameFormatContext` struct, `partition_et_al` helper, and `initialize_given_name` helper.
  Removed both `#[allow(clippy::too_many_arguments)]` suppressions entirely. All tests pass,
  no clippy warnings remain in this slice.

## Summary of Changes

### `partition_et_al` helper
- Extracted et-al partitioning logic from `format_names` into a private helper function.
- Determines which names to show before et-al, whether to use et-al, and which last names to show.
- Encapsulates complex conditional logic for min threshold, effective min, and use_last calculations.

### `NameFormatContext` struct
- Created private `pub(crate)` struct to bundle 7 formatting configuration fields.
- Fields: `display_as_sort`, `name_order`, `initialize_with`, `initialize_with_hyphen`, `name_form`, `demote_ndp`, `sort_separator`.
- Replaced 11 individual parameters in `format_single_name` calls with a single context reference.

### `initialize_given_name` helper
- Extracted given name initialization logic from the `Initials` match arm.
- Handles word separation and initial extraction with proper hyphen preservation.
- Simplifies the code flow and makes the initialization logic reusable and testable.

### Function signature updates
- `format_names`: Now builds `NameFormatContext` once and uses `partition_et_al` helper.
- `format_single_name`: Changed from `pub` to `pub(crate)` (internal-only API); now accepts `&NameFormatContext` instead of 7 loose arguments.

### Test updates
- Added `make_name_format_context` helper for tests to construct contexts easily.
- Updated all 18 `format_single_name` call sites in tests to use the new signature.
- Added explicit `NameForm` import to test module.

### Verification
- All tests pass (no failures, no ignored tests related to this change).
- No clippy warnings in `citum-engine`.
- Code is more readable with clearer parameter grouping and reduced cognitive complexity.
