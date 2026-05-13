---
# csl26-19em
title: Refactor citum-migrate main.rs
status: completed
type: task
priority: normal
created_at: 2026-05-13T23:10:05Z
updated_at: 2026-05-13T23:22:13Z
---

Extract template_diff, cli, bib_postprocess, and citation_validate modules from main.rs (2090 lines → ~400). Refactor validate_and_normalize_inferred_citations into smaller focused functions.

## Summary of Changes

Extracted 4 modules from 2090-line main.rs:
- template_diff.rs (341 lines): LCS-based template variant diff computation
- cli.rs (159 lines): CLI arg parsing + help text
- bib_postprocess.rs (241 lines): bibliography postprocessing, component predicates, type template repair
- citation_validate.rs (161 lines): refactored validate_and_normalize_inferred_citations

validate_and_normalize_inferred_citations split into reject_invalid_inferred_citation + normalize_inferred_citation_contributors, eliminating repeated is_inferred_source checks.

main.rs: 2090 -> 545 lines of logic. All 113 tests pass.
PR: https://github.com/citum/citum-core/pull/690
