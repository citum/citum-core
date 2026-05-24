---
# csl26-fndx
title: Retire Csln prefix; relocate migration IR to citum-migrate::ir
status: completed
type: task
priority: high
created_at: 2026-05-24T11:37:17Z
updated_at: 2026-05-24T12:02:01Z
---

Move CslnStyle/Info/Locale/Node + IR-only block types (TermBlock/VariableBlock/DateBlock/GroupBlock/ConditionBlock/NamesBlock/ElseIfBranch) from citum-schema-style/src/legacy.rs into a new citum-migrate::ir module. Delete the prototype Renderer (citum-schema-style/src/renderer.rs) and render_demo bin (only consumers of CslnNode outside migrate). Drop the misleading CitumNode alias in upsampler.rs. Shared vocabulary (ItemType/Variable/FormattingOptions/date+font enums) stays in citum-schema-style. Full Csln purge across types, re-exports, local bindings, comments. Pre-1.0 breaking rename.

## Todo

- [x] Create crates/citum-migrate/src/ir.rs with moved type defs referencing shared vocabulary from citum_schema
- [x] Add pub mod ir; to citum-migrate/src/lib.rs
- [x] Delete citum-schema-style/src/renderer.rs + src/bin/render_demo.rs + their mod/pub use lines in lib.rs
- [x] Strip IR types from citum-schema-style/src/legacy.rs and the legacy::{...} re-export block in lib.rs (deleted legacy.rs entirely; moved VerticalAlign into template.rs)
- [x] Verify citum-schema facade re-exports (no Csln refs in facade)
- [x] Update citum-migrate template_compiler/{mod,formatting,types,deduplication,node_compiler,bibliography,compilation}.rs
- [x] Update citum-migrate {compressor,compilation,upsampler}.rs + upsampler/mapping.rs
- [x] Update citum-migrate options_extractor/contributors.rs (CslnSubstitute alias dropped; Substitute = output, LegacySubstitute = input)
- [x] Update citum-migrate bin/migrate_all.rs and tests/term_mapping.rs
- [x] Drop CitumNode alias in upsampler.rs:19
- [x] Local-binding + doc-comment sweep (csln_sub→output_sub, csln_nodes→ir_nodes, csln_bib→bib_ir, csln_cit→cit_ir, csln_sort→sort_ir, count_csln_nodes→count_ir_nodes)
- [x] Verify: grep -rn -i csln crates/*.rs returns zero
- [x] cargo fmt && clippy && nextest pass (1344/1344 tests)
- [x] Schema regen + diff review (no docs/schemas/ delta; IR types never reached the published schema)
- [x] Workflow oracle on apa.csl: 51/51 match (18 citations + 33 bibliography)
- [x] Follow-up not needed: legacy.rs deleted entirely (it was 100% IR-only or shared-vocabulary that moved to ir.rs or template.rs)


## Summary of Changes

Retired the `Csln` prototype prefix and relocated the migration IR from `citum-schema-style` to `citum-migrate::ir`. Pre-1.0 breaking rename.

### Type moves (citum-schema-style → citum-migrate::ir)

The entire contents of `citum-schema-style/src/legacy.rs` were either moved into the new `citum-migrate/src/ir.rs` module or repositioned in citum-schema-style. The four prefixed root types became bare names under `ir::`:

- `CslnStyle` → `ir::Style`
- `CslnInfo` → `ir::Info`
- `CslnLocale` → `ir::Locale`
- `CslnNode` → `ir::Node`

All block types (TermBlock, VariableBlock, DateBlock, GroupBlock, ConditionBlock, NamesBlock, ElseIfBranch) plus the vocabulary they reference (ItemType, Variable, FormattingOptions, LabelOptions, LabelForm, DateForm, DateParts, DatePartForm, DateOptions, NamesOptions, EtAlOptions, EtAlSubsequent, NameMode, NameAsSortOrder, DelimiterPrecedes, AndTerm, FontStyle, FontVariant, FontWeight, TextDecoration) live in `citum_migrate::ir` now.

### Stayed in citum-schema-style

- `VerticalAlign` — the only enum genuinely shared with the modern `Rendering` struct. Moved from legacy.rs to template.rs (cohesive home next to its consumer); still re-exported at the citum-schema crate root for `citum-engine`.

### Deletions

- `crates/citum-schema-style/src/legacy.rs` (478 LOC) — entire file gone.
- `crates/citum-schema-style/src/renderer.rs` (459 LOC) — prototype-era renderer, only used by render_demo bin.
- `crates/citum-schema-style/src/bin/render_demo.rs` (60 LOC) — demo bin.
- `pub type CitumNode = citum::CslnNode` alias from upsampler.rs.
- `Substitute as CslnSubstitute` import alias (now bare `Substitute` for the schema output type; the input type from csl_legacy is aliased as `LegacySubstitute`).

### Local-binding rename

All csln_ prefixed locals renamed: csln_nodes→ir_nodes, csln_bib→bib_ir, csln_cit→cit_ir, csln_sort→sort_ir, csln_sub→output_sub, count_csln_nodes→count_ir_nodes. Temp-dir name csln_test_invalid→ir_test_invalid.

### Verification

- `grep -rn -i csln crates/**/*.rs` → 0 hits.
- cargo fmt --check, cargo clippy --all-targets --all-features -D warnings, cargo nextest run (1344/1344) all green.
- Schema regen produced no `docs/schemas/` diff (the IR types never reached the published schema surface).
- APA migration fidelity: 51/51 components match citeproc-js (18 citations + 33 bibliography).

### Net change

-1006 LOC across 26 modified files + 1 new file (citum-migrate/src/ir.rs).
