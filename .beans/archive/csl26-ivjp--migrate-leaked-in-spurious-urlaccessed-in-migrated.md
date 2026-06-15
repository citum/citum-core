---
# csl26-ivjp
title: 'migrate: leaked in. + spurious url/accessed in migrated bibliographies'
status: completed
type: bug
priority: normal
created_at: 2026-06-14T22:25:17Z
updated_at: 2026-06-15T00:13:45Z
parent: csl26-vmcr
---

Split from csl26-ya9b after the bounded genre-echo fix (defect #1) landed. Two template-generation defects remain; both are cross-layer with a broad regression surface (preserving CSL type/host conditionals during template specialization), which is why csl26-ya9b deferred them from the csl26-vmcr bounded PR.

## Remaining defects

- [x] **Leaked `in.`** — the migrated `article-newspaper` template emits an unconditional `term: in` that fires with no host container. Visible in china-information: `Vasari, G. Renaissance Art and Culture. in. Encyclopedia of World History. 2022` (citeproc omits the `in.`). Locus: `crates/citum-migrate/src/template_compiler/`, `passes/suppression.rs`. Gate the term on host/parent-title presence.
- [~] **Spurious url/accessed on non-web types** (split to csl26-e94m) — url/accessed emitted on non-web types across journal-of-advertising-research, early-medieval-europe. Locus: `crates/citum-migrate/src/template_compiler/`, `fixups/template.rs` (already special-cases webpage/accessed). Gate on the type-conditional citeproc uses.

## Repro

```
node scripts/oracle.js styles-legacy/china-information.csl --json --force-migrate
node scripts/oracle.js styles-legacy/journal-of-advertising-research.csl --json --force-migrate
node scripts/oracle.js styles-legacy/early-medieval-europe.csl --json --force-migrate
```

Note: these styles also carry unrelated low-fidelity gaps (missing publisher, dropped volume/pages, name form) per the 2026-06-14 locus audit — compounding defects under a binary threshold.


## Summary of Changes

Defect #1 (leaked `in.`) fixed; defect #2 (url/accessed on non-web) split to **csl26-e94m**.

**Fix:** new converter fixup `crates/citum-migrate/src/fixups/gating.rs` (`gate_leaked_in_term`), applied to every compiled template in `compile_from_xml` (`crates/citum-migrate/src/compilation.rs`). When template specialization flattens a base template into a type-variant, the CSL group binding the `in` preposition to its container (editors/translators/parent-title) is stripped, leaving a bare root-level `Term` the engine renders unconditionally. The fixup re-wraps a root-level `in` with the contiguous run of container companions it introduces — restoring the engine's existing term-only group suppression — or drops it when no companion follows. Terms already inside a group are left untouched (the engine handles them).

**Results (force-migrate oracle):** china-information 32→33 passing bib entries; the spurious `in.` is gone from thesis/report/personal-communication entries. Zero regressions across the top-25 migrate batch, springer-socpsych (37/38) and jar (36/38, "Brown v. Board" unaffected). 4 unit tests; full `just pre-commit` green (1609 tests).

**Why #2 was split:** the url/accessed leak's primary site is the base template, and any base content change propagates through `extends` to type-variants, triggering a latent migrate-vs-engine diff-resolver mismatch that corrupts unrelated entries (jar `legal_case`). That requires an engine-layer fix first — tracked in csl26-e94m.
