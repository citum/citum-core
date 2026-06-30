---
# csl26-zs0f
title: Common Chicago 18 component policy/base
status: completed
type: feature
priority: high
created_at: 2026-06-30T14:29:51Z
updated_at: 2026-06-30T16:30:28Z
parent: csl26-40n4
blocked_by:
    - csl26-fr6f
---

Introduce a hidden common base for Chicago 18 covering contributor formatting, title casing/quotes/italics, page-range format, DOI/URL conventions, periodical/book/chapter/media/archive component semantics, and suppression policy (e.g. personal_communication bibliography suppression). Only where inheritance can express the commonality without forcing the wrong rendered order — author-date and notes ordering stay separate (see csl26-40n4 architecture notes).

## Todo
- [x] Identify genuinely shared component rules from the audit (csl26-fr6f)
- [x] Design hidden base style and extends graph
- [x] Migrate chicago-author-date-18th and chicago-notes-18th onto the shared base
- [x] Verify no regression via report-core.js fidelity across all 4 variants

## Summary of Changes

Introduced a hidden `chicago-18-base` component root carrying only the four options-level rules that are byte-identical across the CMOS author-date and notes heads: `page-range-format: chicago16`, `punctuation-in-quote: true`, `multilingual: romanized-translated`, and `contributors.demote-non-dropping-particle: display-and-sort`. Both `chicago-author-date-18th` and `chicago-notes-18th` now `extends: chicago-18-base` and drop those duplicated keys; T&F-core and shortened-notes-core inherit transitively (unchanged).

Registration: new `embedded/styles/chicago-18-base.yaml`; wired into `get_style_bytes` + `EMBEDDED_STYLE_NAMES` (resolvable via `extends`) but deliberately NOT added to `registry/default.yaml`, so it stays hidden from discovery like the `-core` bases. Added `StyleBase::Chicago18Base` enum variant; regenerated `docs/schemas/style.json`.

Scope deviation from audit Section A (evidence-based): DOI prefix, `pattern.*` message refs, and personal-communication suppression were NOT centralized — direct inspection showed they differ across the two heads (DOI uses note-prose comma joins in notes; personal-communication is bib-suppressed in author-date but a full note citation in notes) or are mere locale message-ID references, not duplicated YAML. Those remain order-layer/head concerns, deferred to csl26-h7oc / csl26-ifhx.

Verification: `report-core.js` fidelity is byte-identical before/after across all four variants (author-date 48/48 cit, 395/500 bib; notes 55/59 cit; shortened 34/34 cit, 46/46 bib; T&F 48/48 cit, 97/98 bib) — pure refactor, zero regression. Updated `all_bases_resolve_cleanly` (component bases skip the citation assertion) and `tier1_bases_have_no_extends_field` (the two heads are now Tier-2; chicago-18-base is the Tier-1 root); added `chicago_18_base_carries_shared_component_options`. Full `just pre-commit` gate green (1691 tests).
