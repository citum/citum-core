---
# csl26-qve8
title: Inferred patent template always emits patent number; oracle omits it
status: completed
type: bug
priority: normal
created_at: 2026-03-27T19:31:22Z
updated_at: 2026-03-28T00:34:37Z
---

The `ensure_inferred_patent_type_template()` fixup in `crates/citum-migrate/src/fixups/media.rs` always includes the patent number field in the inferred template. The oracle (citeproc-js rendering of the CSL source) omits the patent number for the styles affected (karger, thieme, iop, mdpi-like). This causes a bibliography mismatch on any reference that has a patent-number variable populated.

Cluster origin: migrate-research session-3. Classified as engine-gap because the correct behavior depends on the engine respecting a 'suppress patent-number' option that doesn't yet exist in the schema, not on the converter generating wrong YAML.

Affected styles: karger, thieme, iop, mdpi (numeric styles with inferred patent type-variants).

## Summary of Changes

Added legacy_style_uses_number_variable() predicate in media.rs
that walks CSL macros and bibliography layout to detect
number-variable usage.

Replaced style-id guard (springer-socpsych-author-date) with
AST-based detection via the new predicate.

Added patent_number_suppression test for compilation validation.

Inferred patent templates now omit Number.Number component when
the source CSL style never references the number variable.

Affected styles (fixed by regeneration):
- styles-legacy/karger-journals.csl → styles/karger-journals.yaml
- styles-legacy/karger-journals-author-date.csl → styles/karger-journals-author-date.yaml
- styles-legacy/thieme-german.csl → styles/thieme-german.yaml
- styles-legacy/institute-of-physics-numeric.csl → styles/institute-of-physics-numeric.yaml
- styles-legacy/multidisciplinary-digital-publishing-institute.csl (if exists)

Test suite: all 843 tests pass.
