---
# csl26-e94m
title: 'migrate: spurious url/accessed on non-web types (split from ivjp)'
status: in-progress
type: bug
priority: normal
created_at: 2026-06-15T00:09:28Z
updated_at: 2026-06-15T16:16:11Z
parent: csl26-vmcr
---

Split from csl26-ivjp, which fixed the leaked `in.` (defect #1). This is defect #2: migrated bibliographies emit `url` and the `accessed` term/date on non-web types where citeproc-js gates them on `type="webpage post post-weblog"`.

## Repro

```
node scripts/oracle.js styles-legacy/journal-of-advertising-research.csl --json --force-migrate   # interview (Bengio) shows url + bare "accessed"
node scripts/oracle.js styles-legacy/early-medieval-europe.csl --json --force-migrate             # article-journal/report citations show trailing "accessed"
```

Source gate (jar.csl:153-158): url/accessed live under `<if type="webpage"><if variable="URL">`; the converter drops the `type="webpage"` gate, leaking url/accessed into the base template for all types.

## Blocker (why this was split)

The leak's primary site is the **base** template (the interview entry renders via base, not a type-variant). Any base-template content change — wrapping the bare `accessed` term, or suppressing url/accessed — propagates through `extends` to type-variants that inherit the base (e.g. jar `legal_case`), and that triggers a **latent diff-resolver mismatch**: migrate's round-trip validator (`crates/citum-migrate/src/template_diff.rs::apply_template_variant_diff`) accepts a diff, but the engine's resolver (`crates/citum-schema-style/src/template/resolution.rs::resolve_template_variant`) resolves it differently, corrupting the variant's rendering (jar "Brown v. Board of Education" flips pass→fail). Verified 2026-06-14: `in`-only leaves the base untouched and Brown passes; gating base `accessed` corrupts Brown while fixing the interview.

## Diagnosis (2026-06-15)

The url/accessed gate cannot be safely applied within the current `build_final_style` architecture.
The root cause is that diff encoding (`build_type_variants`) is computed in the same pass as fixup
application. Any fixup that touches the base template changes the diff weights for type-variant
encoding. Specifically:

- Gating url/accessed on the base template shifts the base→`legal_case` diff weight, causing
  `bill` to win as a parent variant instead of the base (jar Brown v. Board regression).
- Moving the gate AFTER `build_type_variants` makes it inconsistent with the base stored in
  `Diff.extends` references.
- `engine_validate_variants` (round-trip safety net) validates structural correctness of the diff
  but cannot fix the wrong template content selected via a corrupted diff weight.

This is a systemic problem, not a one-off. Any future fixup that touches the base template will
face the same ordering trap.

## Pivot: spec-first (2026-06-15)

Instead of patching the ordering in-place, this PR delivers a spec documenting the required
refactor: `docs/specs/MIGRATE_FULL_FIRST_ARCHITECTURE.md`.

The spec defines the **Full-first, normalize-later** design:
1. Phase 1 — apply all semantic fixups to Full (standalone) type templates.
2. Phase 2 — run `build_type_variants` once, after all fixups, as a pure compression pass.

## Remaining work (implementation)

- [ ] Restructure `build_final_style` so `build_type_variants` is called only after all fixups.
- [ ] Apply `postprocess_inferred_bibliography` on both inferred and XML-seed-winner paths.
- [ ] Restore `engine_validate_variants` as the Phase 2 round-trip safety net.
- [ ] Implement `gate_web_only_url_accessed` as a Phase 1 fixup per the spec.
- [ ] Verify Brown passes and url/accessed leak is fixed; full batch shows no regression.
