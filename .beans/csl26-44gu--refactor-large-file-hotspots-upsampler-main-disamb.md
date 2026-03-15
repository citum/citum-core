---
# csl26-44gu
title: Refactor large-file hotspots (upsampler, main, disambiguation)
status: in-progress
type: task
priority: deferred
created_at: 2026-03-15T16:03:43Z
updated_at: 2026-03-15T17:18:39Z
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
