---
# csl26-49sj
title: 'Design: conditional number labels (edition 版, volume 第..卷)'
status: completed
type: feature
priority: normal
tags:
    - style
    - fidelity
    - multilingual
created_at: 2026-07-16T15:52:20Z
updated_at: 2026-07-17T11:01:32Z
---

GB/T 7714 renders 版 after edition and 第{n}卷 as a title suffix only when the value is numeric (CSL: choose is-numeric + number form=ordinal + label; CSL-M %s terms like 第%s卷). Citum has no conditional-label mechanism on TemplateNumber and no circumfix number terms. Affects ~12 upstream-corpus entries (7.4:*, 7.2.3:*, 8.13.*, 9.2.1.3:4, plus 康熙字典 volume-title). Candidate designs: TemplateNumber label placement + when-numeric gate, or locale-owned MF2 messages with a $number arg selected per rendering locale. Also decide registered divergences for citeproc quirks (2nd 版 / 5th editors — en ordinal leak into zh, wrong per the standard text).

## Implementation (2026-07-16)

Landed as a typed `when-numeric: <label-form>` gate on `TemplateNumber` (not an MF2 message — see TEMPLATE_V3.md §2.4 for the boundary rationale). The engine resolves the number's general locale term (`GeneralTerm::Edition`/`Volume`) at the given form and wraps the value only when `is_numeric()` (bare digit runs / ranges / lists) — non-numeric values (修订版, 美国卷, 第二卷) render bare. The label text itself is locale-owned: zh-CN's `volume` short term is `第%s卷` (matching the pinned CSL-M term exactly, %s marks the wrap point); `edition`'s short term `版` has no %s and follows the value as a space-separated suffix, matching GB/T's `<number/> <label/>` ordering. This corrected an initial draft that hardcoded 版/第/卷 literally in the shared bilingual base.yaml — caught before landing.

Also fixed: a hand-maintained field allowlist in `citum-schema-style/src/style/diagnostics.rs` (`component_allowed_fields`) duplicates the real struct fields for better error paths; it needed `when-numeric` added to the `number` arm or every embedded-style load panics with a misleading 'unknown property' error.

Numeric upstream-corpus result: 154/203 → 164/203. Official gate (report-core): numeric fidelity 0.808 → 0.845, bib 198/250 → 208/250, SQI 0.985 unchanged. Note style also improved (154→164-equivalent inherited automatically) confirming shared-base design intent.

Remaining in this cluster (deferred, not blocking): 中国科学技术史 volume-title note-cheater-syntax append (1 entry); container-title's own volume-in-chapter wiring (8.3.2:3, needs container editors/title work, folds into csl26-zmod); EN-locale edition ordinal-form + term text (5th/4th 'editors' looks like a citeproc quirk — verify against upstream metadata.json before registering as divergence, not yet done).

## Final numbers (2026-07-17)

Combined with the csl26-zmod structural fixes (landed concurrently) plus one additional conversion fix of my own (from_document_ref in scholarly.rs never wired legacy.edition/legacy.page into Monograph.edition/pages for map/figure/graphic/periodical/collection/document types -- a real, separate gap, not part of the original when-numeric design but caught while investigating why map entries were missing editions entirely): numeric upstream corpus **190/203**.

Remaining 13 failures split into:
- **4 entries (7.4:5, 8.2.2:6, 8.3.2:4, 8.3.2:5)**: NOT a citeproc/oracle divergence as I initially suspected -- verified both sides render the same 'editor(s)' term text; the only diff is ordinal-vs-plain number form (5th vs 5). Root cause: TemplateNumber.form: Option<NumberForm> already has an Ordinal variant in the schema, but the engine (crates/citum-engine/src/values/number.rs) never reads self.form at all -- ordinal rendering is a schema-only no-op today. Filed as its own feature bean rather than implemented here (locale-aware ordinal suffix rules are a real feature, not a one-line fix, and out of proportion for 4/203 entries in this PR).
- **9 entries**: genuine remaining structural/data gaps, filed as csl26-ra71 (wave-3).

## Summary of Changes

Implemented `TemplateNumber.when-numeric: <label-form>` (typed gate, not MF2 — see TEMPLATE_V3.md §2.4). Engine resolves the number's locale term via GeneralTerm and wraps the value with %s-circumfix or space-suffix semantics, only when is_numeric(). Fixed a latent diagnostics.rs allowlist bug found along the way. Numeric upstream corpus 154→164/203 from this bean's own scope; combined with csl26-zmod, 190/203 overall. Remaining ordinal-form gap filed as csl26-g49a.
