---
# csl26-44gu
title: Refactor large-file hotspots (upsampler, main, disambiguation)
status: completed
type: task
priority: deferred
created_at: 2026-03-15T16:03:43Z
updated_at: 2026-03-15T20:02:29Z
---

Three files not touched by the simplify wave still exceed 1000 lines and carry scoped clippy suppressions:
- citum-migrate/src/upsampler.rs (1,983 lines)
- citum-migrate/src/main.rs (1,492 lines)
- citum-engine/src/processor/disambiguation.rs (1,022 lines)

Each has a FIXME comment referencing this bean. Address when prioritized.

## Work Plan

Executing on this PR branch. 35 suppressions total.

**Test files (7) — convert FIXME to permanent allows:**
- [ ] citum-analyze/src/batch_test.rs:20
- [ ] citum-engine/src/processor/tests.rs:1022, 2194
- [ ] citum-engine/tests/citations.rs:184
- [ ] citum-engine/tests/i18n.rs:634
- [ ] citum-migrate/tests/substitute_extraction.rs:14
- [ ] citum-migrate/tests/term_mapping.rs:199

**Production code (28) — actual refactors:**
- [ ] citum-schema-style: locale/mod.rs, presets.rs, reference/conversion.rs
- [ ] values/: contributor/labels, contributor/mod, contributor/names (x2), contributor/substitute, date, mod, number, variable
- [ ] render/: component.rs, rich_text.rs
- [ ] ffi/biblatex.rs
- [ ] citum-cli/main.rs
- [ ] citum-analyze/: analyzer.rs (x2), ranker.rs
- [ ] citum-migrate/: analysis/citation.rs, fixups/media.rs, passes/deduplicate.rs, passes/grouping.rs, template_compiler/compilation.rs
- [ ] citum-migrate/src/main.rs (x3)
- [ ] citum-migrate/src/upsampler.rs

## Summary of Changes

All FIXME suppressions removed from production code via helper extraction across multiple files:

- citum-analyze/src/analyzer.rs: extracted per-tag and per-section helpers
- citum-analyze/src/ranker.rs: extracted phase helpers
- citum-migrate/src/analysis/citation.rs: moved inner fns to module level
- citum-migrate/src/passes/grouping.rs: extracted 3 case helpers
- citum-migrate/src/passes/deduplicate.rs: extracted suppress_component_if_seen
- citum-migrate/src/fixups/media.rs: extracted apply_legal_case_additions
- citum-migrate/src/template_compiler/compilation.rs: extracted 2 impl methods
- citum-migrate/src/upsampler.rs: extracted apply_name_child_options
- citum-schema-style/src/locale/mod.rs: extracted role/locator term helpers
- citum-schema-style/src/presets.rs: extracted 3 sub-matchers
- citum-schema-style/src/reference/conversion.rs: extracted 11 per-arm functions
- citum-migrate/src/main.rs: extracted 11 helpers to clear main/compile_from_xml/apply_type_overrides

Zero FIXME suppressions remain. 712 tests pass. Workspace lints fully enforced.
