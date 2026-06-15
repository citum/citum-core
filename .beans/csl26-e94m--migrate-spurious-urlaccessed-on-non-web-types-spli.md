---
# csl26-e94m
title: 'migrate: spurious url/accessed on non-web types (split from ivjp)'
status: todo
type: bug
priority: normal
created_at: 2026-06-15T00:09:28Z
updated_at: 2026-06-15T00:09:38Z
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

## Plan

- [ ] Reproduce and fix the migrate-vs-engine diff-application mismatch so a base/parent change cannot corrupt `extends`-based type-variants. Likely: align `apply_template_variant_diff` with `resolve_template_variant`, or harden migrate's round-trip check to validate via the engine resolver so only engine-safe diffs are emitted.
- [ ] With the diff resolver hardened, gate url + the `accessed` term/date on web types only (`webpage`, `post`, `post-weblog`), per the source `type=` conditional. The `crates/citum-migrate/src/fixups/gating.rs` infrastructure (companion run + group wrapping) and `fixups/template.rs` webpage special-casing are the starting points.
- [ ] Verify the two repros above no longer show spurious url/accessed, and the full migrate batch (`oracle-migrate-batch.js`) shows no regression (especially jar `legal_case`).
