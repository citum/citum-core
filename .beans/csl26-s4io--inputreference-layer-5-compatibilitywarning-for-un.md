---
# csl26-s4io
title: 'InputReference Layer 5: CompatibilityWarning for unknown class'
status: todo
type: task
priority: normal
created_at: 2026-05-16T00:15:48Z
updated_at: 2026-05-16T00:15:48Z
parent: csl26-1bdr
---

Surface unknown-class references through a `CompatibilityWarning` channel so the engine emits soft-degrade messaging instead of silently falling through to an empty render.

## Background

PR #717 landed the discriminator cutover (Layers 1–4) and explicitly deferred Layer 5. Today, `InputReference::ref_type()` for an `Unknown` class returns the raw kebab-case class string — which the engine has no template branch for — so renders silently produce empty output. The current code has a `debug_assert!` + `TODO(csl26-1bdr)` marker pinning this gap.

## Approach

- Add a `CompatibilityWarning` enum / channel that the engine pipeline can emit during processing.
- When an unknown-class reference is processed, emit a `UnknownReferenceClass { class, ref_id }` warning.
- Hub / CLI surface: render the warnings alongside output (CLI: stderr; WASM: structured result).
- Replace the `debug_assert!` in `ref_type()` for Unknown with a proper warning emission path.

## Acceptance criteria

- [ ] Unknown-class references emit a structured warning during render.
- [ ] CLI prints warnings on stderr without breaking output.
- [ ] WASM bridge exposes warnings in the structured result.
- [ ] Round-trip preservation of unknown-class data is unchanged.
- [ ] Tests cover: unknown class in citation list, unknown class in bibliography, unknown class in both.
