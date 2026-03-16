---
# csl26-fk0w
title: Pre-release style + engine co-evolution wave
status: in-progress
type: epic
priority: normal
created_at: 2026-03-16T10:34:43Z
updated_at: 2026-03-16T12:11:27Z
---

Track A: engine fixes for cross-portfolio gaps (volume-pages, DOI suppression, editor name-order). Track B: upgrade outlier styles (springer-physics-brackets, royal-society-of-chemistry, MLA). Track C: promote chicago-author-date from experimental. PR branch: style-evolve-pre-release-wave.

## Progress Log

### Session 2026-03-16 #2
**Completed:**
1. **OSCOLA + OSCOLA-no-ibid**: Fixed publisher bleed in citations (removed from citation template, kept in bibliography). Quality improvement pending oracle re-run.
2. **American Geophysical Union**: Fixed locator separator — added ", " prefix to locator variable in citations. Fixes `(Kuhn, 1962p. 23)` → `(Kuhn, 1962, p. 23)`.

**In Progress:**
- 75 styles use `variable: locator` in citations; AGU fix is cross-portfolio pattern candidate
- Identified that many author-date styles (APA 7th, MLA, Chicago, etc.) may benefit from locator prefix review
- Need to verify if this is an engine-level default or style-level oversight

**Deferred (complex/multi-step):**
- RSC: 7 bibliography failures — thesis, legal case, DOI URL formatting (div-002 applies)
- MLA: ~1-2 failures (need oracle run to confirm)
- OSCOLA name abbreviation issue (Thomas S vs TS) — engine bug csl26-q4k2 filed

**Engine Work Identified:**
- csl26-q4k2: initialize-with empty string causes name abbreviation when form: long is used
- 75 styles use variable: locator; pattern suggests this might need cross-portfolio audit
- Potential cross-portfolio locator prefix improvements for author-date styles (APA, MLA, Springer variants)

## RSC (royal-society-of-chemistry)
- Fixed: thesis type-template with genre label
- Fixed: legal_case type-template (number+title pair, year, volume; removed authority/publisher bleed)
- Fixed: title suppression for entry-encyclopedia, broadcast, interview, personal_communication
- Fixed: publisher suppression for entry-encyclopedia
- Result: 32/33 bibliography, 18/18 citations
- Deferred: DOI rendering for article-journal (missing-feature)

## AGU + MLA
- AGU: 33/33 bibliography — added interview (full date, medium bracket, Retrieved from URL) and patent (and:symbol, full date, patent number) type-templates
- MLA: 18/18 citations — fixed by engine fix (see below)

## Engine Fix (processor-defect)
- crates/citum-engine: per-cite suffix was silently dropped in author-only grouped citations (MLA non-integral path). Fixed in render_fallback_grouped_citation_with_format — suffix now applied when item_parts is empty. Commit: 366167c. Unlocks: any author-date style with no year in citation template (MLA et al.)

## Core Quality Gate
- 146 styles, all at fidelity 1.0
