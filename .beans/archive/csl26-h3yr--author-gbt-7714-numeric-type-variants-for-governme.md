---
# csl26-h3yr
title: Author GB/T 7714 numeric type-variants for government/legal/media reference classes
status: completed
type: task
priority: normal
tags:
    - gb-t
    - style
    - fidelity
    - rendering
created_at: 2026-07-24T14:25:51Z
updated_at: 2026-07-24T15:37:48Z
---

gb-t-7714-2025-numeric (and the shared gb-t-7714-2025-base bibliography grammar it inherits) has no bibliography.type-variants entry for several ref_type() strings that legitimately occur in the broader test corpus: motion-picture, broadcast, legal-case, treaty, hearing, regulation, statute, and interview (monograph). Reference.ref_type() and the CSL-JSON->Citum conversion were verified correct for all of these via the conversion-layer pre-flight (cargo run --bin citum -- convert refs ... --from csl-json); the defect is entirely on the style-authoring side.

With no matching type-variant key, these classes fall through to gb-t-7714-2025-base.yaml's un-keyed default bibliography.template, which was authored as a bare component list with almost no inter-component delimiters/suffixes (it appears to have been intended as a components library, always meant to be overridden by a type-variant, not rendered directly). The result is badly garbled bibliography output: no punctuation between fields, wrong/missing type-code brackets (e.g. literal medium value shown as "[Film]"/"[Television]" instead of the gb-t-7714-type-code message producing "[Z/film]"), and misplaced volume/page numbers.

Evidence (from tests/fixtures/references-expanded.json, exercised via 'node scripts/report-core.js --style gb-t-7714-2025-numeric'):
- ITEM-22 (motion_picture/film) -> ref_type 'motion-picture'
- ITEM-23 (broadcast, container broadcast-program) -> ref_type 'broadcast'
- ITEM-24 (interview) -> class monograph, type 'interview'
- ITEM-20 (legal_case) -> ref_type 'legal-case'
- TLIB-SEL-TREATY-1 -> ref_type 'treaty'
- TLIB-SEL-BILL-1 (bill with title+authority, intentionally routes to 'hearing' per docs/specs/CSL_TYPE_CONVERSION_CONTRACT.md) -> ref_type 'hearing'
- TLIB-SEL-HEARING-1 -> ref_type 'hearing'
- TLIB-SEL-REGULATION-1 -> ref_type 'regulation'
- TLIB-SEL-LEGISLATION-1 (legislation, canonicalizes to 'statute' per the conversion contract) -> ref_type 'statute'

None of these 9 items are in the style's GATED native corpus (tests/fixtures/test-items-library/gb-t-7714-2025.json, 203 items) or count toward the CI fidelity gate (gb-t-7714-2025-numeric is not in scripts/report-data/core-quality-baseline.json) -- confirmed 'just check-core-quality' passes on main without this fix. This is a real fidelity gap against the broader diagnostic corpus for an embedded-core style, not a CI blocker.

Scope: author type-variants for these ref_type() strings (grouping compatible ones, e.g. legal-case+treaty share reporter/volume/page shape; hearing+regulation+statute share code/section/volume shape) following the tune loop in docs/guides/STYLE_WORKFLOW_EXECUTION.md. Use the GB/T 7714-2025 standard's own worked examples (referenced throughout docs/adjudication/DIVERGENCE_REGISTER.md's div-014/div-015 entries) as authority for any of these classes the standard itself covers; where the standard doesn't cover a class (these are US legal/media reference kinds with no GB/T-native worked example), citeproc-js's actual oracle output is the best available fidelity target.

Also worth checking as a smaller side finding: the software/graphic/article,dataset,preprint/webpage,post,post-weblog type-variants share a 'date: accessed (suffix period) then variable: url (no prefix)' pattern that silently drops the joining period when accessed is absent (fixed for 'software' specifically in csl26-gc43; article,dataset,preprint/graphic/webpage,post,post-weblog are unverified for this same latent gap since no failing fixture currently exercises an accessed-less item there).

Confirmed via the full report-core.js run (all styles, fresh cache): `gb-t-7714-2025-author-date.yaml` has its own separate, non-shared `software` type-variant (line ~729) with the identical accessed-suffix/url-no-prefix bug fixed in `csl26-gc43` for the numeric style's copy. Still shows `（2015）http://...` (missing period) for TLIB-SEL-SOFTWARE-1. Not touched by csl26-gc43 (out of scope — that PR only covers gb-t-7714-2025-numeric, and author-date is diagnostic-only per verification-policy.yaml, not gated). Confirmed this is pre-existing, not a regression from csl26-gc43's changes.

## Summary of Changes

Authored the 8 missing `bibliography.type-variants` entries in `gb-t-7714-2025-base.yaml`: `legal-case`, `treaty`, `regulation,statute` (shared shape), `hearing` (also covers `bill`, which the conversion layer canonicalizes to `hearing`), `motion-picture`, `broadcast`, `interview`. All 9 previously-garbled oracle entries ([20],[22],[23],[24],[40],[43],[44],[46],[47]) now match byte-for-byte.

While verifying `regulation`, found and fixed a real engine bug (not a style gap): `Reference::container_title()` in `crates/citum-schema-data/src/reference/accessors.rs` had match arms for `LegalCase`, `Statute`, and `Treaty` but was missing `Regulation`, so `regulation`-type references silently dropped their `code` field (e.g. "C.F.R.") from `container_title()`. Added the missing arm — same shape of bug as the `medium()`/`Software` gap fixed in `csl26-gc43`.

Result: `gb-t-7714-2025-numeric` bibliography 249/250 (was 240/250); only the CSTR duplicate-identifier item `[178]` remains, deferred to `csl26-ia43` (blocked on external reviewer confirmation, out of scope here). Gated 203-item native corpus unaffected (unchanged raw 193/203, i.e. adjusted 202/203 — same as before this change). `docs/compat.html` regenerated to reflect current numbers. Full suite: 2181/2181 passing, clippy clean.

The side-finding about accessed-suffix/url-no-prefix punctuation in `article,dataset,preprint`/`graphic`/`webpage,post,post-weblog` was not touched — none of the new type-variants exercised that pattern, so it remains a separate, not-yet-filed follow-up if anyone hits it.
