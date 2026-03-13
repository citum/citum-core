---
# csl26-rsw1
title: Clean up clippy warning hotspots with rust-simplify
status: todo
type: task
priority: normal
created_at: 2026-03-13T22:44:40Z
updated_at: 2026-03-13T22:44:40Z
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

- [ ] Simplify `crates/citum-engine/src/processor/document/mod.rs`
  - [ ] reduce `process_document`
  - [ ] reduce `prepare_note_citation_state`
- [ ] Simplify `crates/citum-engine/src/processor/rendering.rs`
  - [ ] reduce `render_grouped_citation_with_format`
  - [ ] reduce `process_template_request_with_format`
- [ ] Simplify `crates/citum-engine/src/values/contributor.rs`
  - [ ] reduce `values`
  - [ ] reduce `format_names`
  - [ ] reduce `format_single_name`
- [ ] Simplify `crates/citum-engine/src/values/date.rs`
  - [ ] reduce `values`
- [ ] Simplify `crates/citum-engine/src/values/number.rs`
  - [ ] reduce `values`
- [ ] Simplify `crates/citum-engine/src/values/title.rs`
  - [ ] reduce `values`
- [ ] Simplify `crates/citum-engine/src/values/variable.rs`
  - [ ] reduce `values`
- [ ] Simplify `crates/citum-engine/src/io.rs`
  - [ ] reduce `load_bibliography_with_sets`
- [ ] Re-run `cargo clippy --all-targets --all-features -- -D warnings` once the
      current target slice is complete
- [ ] Archive or update this bean when the warning cleanup frontier changes

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
