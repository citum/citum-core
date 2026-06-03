---
# csl26-ynra
title: 'Integral citation clusters: terminology spec + sentence-start signal'
status: completed
type: feature
priority: normal
created_at: 2026-06-03T21:08:46Z
updated_at: 2026-06-03T21:55:51Z
---

Document the citation/CitationItem/CitationMode terminology; add sentence_start bool to Citation for sentence-initial capitalization; wire the CapitalizeFirst transform in the engine; write INTEGRAL_CITATION_CLUSTERS.md spec. Affix-placement configurability is deferred as a future enhancement. Djot auto-detection of sentence position is a separate follow-up.

## Summary of Changes

- Added `sentence_start: bool` to `Citation` (citum-schema-data) and `sentence_start: Option<bool>` to `CitationOccurrence` (api/types.rs) so server/document callers can pass the signal through
- Engine applies locale-aware `CapitalizeFirst` via `resolve_text_case` + `apply_text_case_markup_aware` when `sentence_start` is set
- Wrote `docs/specs/INTEGRAL_CITATION_CLUSTERS.md` (Active) — terminology glossary, list-like integral default, affix placement contract, sentence_start design with natbib/biblatex prior art
- 4 new tests (3 engine BDD + 1 schema round-trip), 1498/1498 passing
- PR #874 merged with Copilot review integrated; CI green
