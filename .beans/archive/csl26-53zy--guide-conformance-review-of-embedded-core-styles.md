---
# csl26-53zy
title: Guide-conformance review of embedded core styles
status: completed
type: task
priority: high
created_at: 2026-06-20T19:03:06Z
updated_at: 2026-06-21T10:50:54Z
---

Audit all 15 embedded core styles in `crates/citum-schema-style/embedded/styles/` against their published guides: verify metadata/source URLs, then compare Citum YAML output vs the citeproc oracle snapshot vs the published guide. Three-way method, finding buckets (A=Citum wrong, B=CSL/citeproc wrong, C=fixture mistyped, D=inaccessible/ambiguous, E=engine).

Full detail — method, per-style conformance tables, metadata inventory, verified T&F PDF mapping — lives in the audit report: `docs/architecture/audits/2026-06-20_STYLE_GUIDE_CONFORMANCE.md`. PR #946.

## Summary of Changes

First sweep complete — all 15 styles reviewed; clean low-risk fixes landed, structural/engine residuals deferred to the follow-up beans below.

- **Engine**: pull a comma (not just a period) inside a closing quote in bibliographies when `punctuation-in-quote` is set.
- **Schema**: optional `text-case` on `RoleLabel` (renders IEEE `Eds.` from the locale's lowercase term).
- **Styles** (each verified vs guide + citeproc):
  - IEEE — chapter/thesis/conference variants, et-al 8→7, terminal period, serial comma, capitalised `Eds.`
  - APA 7 — journal article titles roman (not italic)
  - Chicago author-date — journal `Journal Vol (Iss):` punctuation
  - AMA 11 — period before `doi:`; short `eds.` editor label
  - MLA 9 — `, vol. N` label + journal DOI comma (one golden updated)
  - Elsevier Vancouver — chapter `In:`; Elsevier Harvard already conformant
  - Springer ×3 — chapter `In:`; fixed brackets duplicate page range
  - T&F NLM — author period; `In: <names>, editors. <Book>`; CSE/NLM doc URLs corrected against the official PDFs (`tf_c.pdf`, `tf_nlm.pdf`)
- Verified with `just pre-commit` (fmt, clippy, 1659 tests).

## Follow-ups

- `csl26-28ag` — T&F trio structural conformance (incl. Style-F sentence-case decision)
- `csl26-6qv3` — embedded style structural residuals (Chicago-AD, Elsevier with-titles, Chicago notes)
- `csl26-maim` — cross-cutting render residuals (substitute `(eds.)`, entry-suffix DOI/URL policy, disambiguation, sentence-case proper nouns, page-range dash)
