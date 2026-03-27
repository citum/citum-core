---
# csl26-qve8
title: Inferred patent template always emits patent number; oracle omits it
status: todo
type: bug
priority: normal
created_at: 2026-03-27T19:31:22Z
updated_at: 2026-03-27T19:31:22Z
---

The `ensure_inferred_patent_type_template()` fixup in `crates/citum-migrate/src/fixups/media.rs` always includes the patent number field in the inferred template. The oracle (citeproc-js rendering of the CSL source) omits the patent number for the styles affected (karger, thieme, iop, mdpi-like). This causes a bibliography mismatch on any reference that has a patent-number variable populated.

Cluster origin: migrate-research session-3. Classified as engine-gap because the correct behavior depends on the engine respecting a 'suppress patent-number' option that doesn't yet exist in the schema, not on the converter generating wrong YAML.

Affected styles: karger, thieme, iop, mdpi (numeric styles with inferred patent type-variants).
