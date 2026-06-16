---
# csl26-2e3d
title: 'Strip spurious suppress: false from migrated full templates'
status: in-progress
type: task
priority: high
tags:
    - migrate
    - authorability
    - cleanup
    - template
created_at: 2026-06-16T12:54:57Z
updated_at: 2026-06-16T12:54:57Z
---

Occurrence-compiler (template_compiler/compilation.rs:477) sets suppress=Some(false) as a 'visible by default' marker. It leaks into serialized YAML (~24 noise lines in ACME) because suppress uses skip_serializing_if=Option::is_none, so only Some(false) emits. Some(false) is semantically identical to None.

## Fix
Add normalize_visible_suppress(components) to passes/suppression.rs: recurse groups, set suppress=None wherever Some(false); leave Some(true)/None.

Call it at the END of assembly.rs::finalize_bibliography_variants (after build_type_variants/diff encoding) on new_bib, each full type_templates value, and the citation template. MUST run downstream of diff encoding so modify: suppress:false blocks (legal_case date+volume un-suppress) survive.

## Done when
- [ ] normalize_visible_suppress added with unit test
- [ ] called post-variant-build on full template lists only
- [ ] ACME: full-template suppress:false lines gone; modify: suppress:false preserved
- [ ] suppress:true unchanged; oracle output identical (serialization-only)
- [ ] just pre-commit green; amended into 74706439

Folds into PR #932.
