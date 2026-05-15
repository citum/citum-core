---
# csl26-powf
title: 'Address Copilot review on PR #717 discriminator impl'
status: completed
type: task
priority: high
created_at: 2026-05-15T23:42:08Z
updated_at: 2026-05-15T23:56:01Z
parent: csl26-1bdr
---

Resolve 16 Copilot inline comments on PR #717 (csl26-odgh discriminator cutover).

## Tier A — MUST-FIX (correctness / API hazards)
- [x] 1. Delete crates/citum-schema-data/src/reference/discriminator/ (parallel public types)
- [x] 2. Demote 16 pub fields on InputReference to pub(crate) (stale-fields footgun)
- [x] 3. Unknown::ref_type() — debug_assert + sentinel + Layer-5 TODO
- [x] 4. SharedReferenceFields::from_serializable — debug_assert on swallow

## Tier B — SHOULD-FIX (low-risk quality)
- [x] 5. as_event / as_audio_visual — add .as_ref() for consistency
- [x] 6. from_known — avoid unconditional value.clone()
- [x] 7. from_known — replace io error with de::Error::custom / unreachable
- [x] 8. Deserialize — use de::Error::duplicate_field uniformly
- [x] 9. ReferenceClass::Unknown — doc the serde(skip) round-trip via UnknownClassData
- [x] 10. set_id on Unknown — doc string-id constraint
- [x] 11. Transitional Monograph(...) etc constructors — block-level TODO(csl26-1bdr) marker
- [x] 12. EMPTY_FIELD_LANGUAGES — investigated (HashMap::new not const, kept LazyLock with rationale doc)

## Test rigor
- [x] Strengthen assertions around extension_mut → accessor consistency
- [x] Strengthen assertions around unknown-class round-trip (id, ref_type, ser/de)
- [x] Strengthen assertions around duplicate-field error shape (canonical serde form)

## Gate + ship
- [x] cargo fmt --check && cargo clippy -D warnings && cargo nextest run
- [x] Regenerate docs/schemas/bib.json (semantic no-op; jq -S confirms identical)
- [ ] Squash with jj, push, watch CI green
- [ ] Reply to / resolve all 16 Copilot threads
- [ ] Open follow-up beans: perf hot-path (#13/14/15), Layer-5 plumbing

Parent: csl26-1bdr

## Summary of Changes

All 16 Copilot inline comments resolved on PR #717:

- Deleted the entire 596-line `crates/citum-schema-data/src/reference/discriminator/` scaffolding module that exposed parallel public `InputReference` / `ClassExtension` / `ReferenceClass` types. This single change resolves comments #3251089378 and #3251089506.
- Demoted 16 shared `pub` fields on `InputReference` to `pub(crate)`. Cross-crate verification showed no production reader of the duplicated fields; accessors are now the only public read path.
- `ref_type()` for `Unknown` class adds a `debug_assert!` that the class string is not a KNOWN name, and a `TODO(csl26-1bdr)` for Layer 5 CompatibilityWarning plumbing.
- `SharedReferenceFields::from_serializable` adds `debug_assert!` so silent default-on-serialize-failure regressions surface in debug builds.
- `as_event` / `as_audio_visual` now call `.as_ref()` for consistency with the other 17 accessors.
- `from_known` matches `&value` to avoid the per-reference deep clone; defensive non-object branch uses `de::Error::custom` instead of `Error::io(InvalidData)`.
- New `duplicate_field_error<E>` helper produces the canonical `duplicate field \`X\`` shape for both the class and non-class duplicate-key paths, restoring uniform error semantics.
- `ReferenceClass::Unknown` gains a doc note explaining the `#[serde(skip)]` round-trip via `UnknownClassData::class`.
- `set_id` gains documentation on the string-id wire constraint for Unknown class.
- Transitional PascalCase `InputReference::Monograph(...)` constructors are now preceded by a block-level note + `TODO(csl26-1bdr)` marker pinning their removal to the parent epic. Full `#[deprecated]` attribute deferred (would force `#[allow(deprecated)]` across 79 call sites; agreed scope per plan).
- `EMPTY_FIELD_LANGUAGES` gains a doc explaining why `LazyLock` is required (HashMap::new is not const).

Test rigor strengthened: 13 tests now cover the discriminator surface (up from 5), with structural assertions on serialized JSON (no `contains()` on short substrings), canonical serde-error-shape checks, accessor/extension agreement across 6 class variants, set_id sync on both known and unknown classes, and a ref_type sentinel non-collision guard.

Gate: cargo fmt --check + cargo clippy --all-targets --all-features -- -D warnings + cargo nextest run = 1282 passed. Schema regen produces a key-reordered but semantically identical `docs/schemas/bib.json` (verified via `jq -S` diff).

Deferred follow-ups (will be tracked in separate beans):
- Perf hot-path (Copilot #405, #475, #491) — drop the `to_value` round-trip in construction/serialize/visit paths; needs benchmark data before investment.
- Layer 5 `CompatibilityWarning` plumbing for unknown-class soft-degrade UX (out of scope per original PR body, now explicitly marked in code with TODO).
