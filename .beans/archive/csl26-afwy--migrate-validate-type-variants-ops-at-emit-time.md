---
# csl26-afwy
title: 'migrate: validate type-variants ops at emit time'
status: completed
type: bug
priority: high
created_at: 2026-06-10T16:56:45Z
updated_at: 2026-06-10T21:39:57Z
parent: csl26-vmcr
---

Cluster C1 from docs/architecture/audits/2026-06-10_MIGRATE_RANDOM_SAMPLE_BASELINE.md. citum-migrate emits bibliography.type-variants operations that reference components absent from the base template; the processor hard-fails the entire style at render (template variant operation matched no component). Evidence: zeitschrift-fur-fantastikforschung (interview), american-mathematical-society-label (patent). Fix in crates/citum-migrate: validate variant ops against the emitted base template; drop or repair invalid ops and record the decision in the evidence sidecar. Add regression tests. Converter must never emit YAML the processor rejects.

## Summary of Changes

Root cause was sharper than emit-time validation: wrapper emission (lineage apply_to_migrated_style) kept type-variant Diff ops derived against the childs standalone default template, but the engine resolves a no-extends Diff against the parents same-selector variant when one exists (embedded modern-language-association defines interview; elsevier-with-titles defines patent) — anchors missing, processor hard-fails the whole style.

Fix: new public citum_schema::template::resolve_local_template_variants (wraps the engines own resolution with no inherited context); the wrapper path materializes every Diff variant as Full before attaching extends. Diffs remain only in standalone emissions where their base is the local template.

Evidence: both hard failures now render — zeitschrift-fur-fantastikforschung 47/58 (81%), american-mathematical-society-label 25/58 (43%). Sentinels exact (200/200, 375/378); 1546/1546 tests; JSON schemas unchanged.
