---
# csl26-odgh
title: Implement InputReference class discriminator design
status: in-progress
type: task
priority: high
created_at: 2026-05-15T19:00:04Z
updated_at: 2026-05-15T20:30:21Z
blocked_by:
    - csl26-1bdr
---

Implementation of `docs/specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md` (v0.3).

Pre-1.0 hard cut originally targeted replacing the closed `#[serde(tag = "class")]` enum with a shared-base struct + class-specific overlay. Current Layer 1 keeps the existing enum storage/API inside `citum-schema-data` so Layers 2-4 do not get pulled into this session, but replaces the public wire/schema boundary with a hand-written flat discriminator dispatcher and restores strict class payload deserialization.

## Layered change stack

- [x] **Layer 1 — schema-data refactor.** Complete for the scoped single-session cut: live `InputReference` now has custom flat-map `Deserialize`/`Serialize`, all 18 known classes dispatch through explicit `class` values, unknown classes capture into `UnknownClassData`, `ReferenceClass`/`ClassExtension`/`as_<class>()`/`unknown_class()` accessors are present, `deny_unknown_fields` is restored across reference payloads/helpers, and `InputReference` has a custom schema composition with per-class `class` const branches plus `unevaluatedProperties: false`. Verification: `cargo fmt --check`; `cargo clippy -p citum-schema-data --all-features --all-targets -- -D warnings`; `cargo nextest run -p citum-schema-data --all-features` (108 passed). See `.ai-intents/INTENT-2026-05-15-1500-layer1-schema-data.md` for handoff state.
- [ ] **Layer 2 — citum-io biblatex constructors.** Deferred.
- [ ] **Layer 3 — citum-migrate constructors.** Deferred.
- [ ] **Layer 4 — engine call-site rewrites.** Deferred.
- [ ] **Layer 5 — `CompatibilityWarning` plumbing.**
- [ ] **Layer 6 — forward-compat tests + snapshot.** Flip row 02b; close row 07.
- [ ] **Layer 7 — schema artifact regeneration.**
- [ ] **Layer 8 — schema-alignment tests.** Expand beyond the Layer 1 schema smoke test.

## Workflow

- Single PR; multi-session.
- jj change stack per `docs/guides/JJ_AI_CHANGE_STACK.md`.
- `.ai-intents/` tracks session handoff state and must be removed before merge-ready publication.
- Pre-commit gate run manually before every push (jj skips git hooks).

Spec: `docs/specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md`
Parent design bean (related): `csl26-1bdr`
