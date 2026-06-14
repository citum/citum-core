---
# csl26-ivjp
title: 'migrate: leaked in. + spurious url/accessed in migrated bibliographies'
status: todo
type: bug
priority: normal
created_at: 2026-06-14T22:25:17Z
updated_at: 2026-06-14T22:25:17Z
parent: csl26-vmcr
---

Split from csl26-ya9b after the bounded genre-echo fix (defect #1) landed. Two template-generation defects remain; both are cross-layer with a broad regression surface (preserving CSL type/host conditionals during template specialization), which is why csl26-ya9b deferred them from the csl26-vmcr bounded PR.

## Remaining defects

- [ ] **Leaked `in.`** — the migrated `article-newspaper` template emits an unconditional `term: in` that fires with no host container. Visible in china-information: `Vasari, G. Renaissance Art and Culture. in. Encyclopedia of World History. 2022` (citeproc omits the `in.`). Locus: `crates/citum-migrate/src/template_compiler/`, `passes/suppression.rs`. Gate the term on host/parent-title presence.
- [ ] **Spurious url/accessed on non-web types** — url/accessed emitted on non-web types across journal-of-advertising-research, early-medieval-europe. Locus: `crates/citum-migrate/src/template_compiler/`, `fixups/template.rs` (already special-cases webpage/accessed). Gate on the type-conditional citeproc uses.

## Repro

```
node scripts/oracle.js styles-legacy/china-information.csl --json --force-migrate
node scripts/oracle.js styles-legacy/journal-of-advertising-research.csl --json --force-migrate
node scripts/oracle.js styles-legacy/early-medieval-europe.csl --json --force-migrate
```

Note: these styles also carry unrelated low-fidelity gaps (missing publisher, dropped volume/pages, name form) per the 2026-06-14 locus audit — compounding defects under a binary threshold.
