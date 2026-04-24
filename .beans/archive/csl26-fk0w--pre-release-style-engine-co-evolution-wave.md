---
# csl26-fk0w
title: Pre-release style + engine co-evolution wave
status: completed
type: epic
priority: normal
created_at: 2026-03-16T10:34:43Z
updated_at: 2026-04-24T12:13:54Z
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

## Follow-up Tracking
- The low-risk rendering optimization slice landed under archived child `csl26-3oq0`.
- Deferred next slice: `csl26-d59c` tracks benchmarked disambiguation hot-path optimization work that was explicitly excluded from `csl26-3oq0`.

## Session 2026-03-27

**Completed:**
1. **springer-physics-brackets**: Fixed broadcast type-variant — changed `title: primary` → `title: parent-serial` in bibliography. Renders container-title (show name) instead of episode title. Now 33/33 bibliography, fidelity 1.0.
2. **modern-language-association (MLA)**: Fixed interview type-variant — suppressed auto `(Interviewer)` label via `role.omit`, changed prefix from `. Interview by ` to `Interview by ` (triggers punctuation-in-quote), changed date form to `day-month-abbr-year`, added URL and medium variables. Now 33/33 bibliography (was 32/33).
3. **chicago-author-date**: Promoted to Production in TIER_STATUS.md — core fixture 100% fidelity (18/18 citations, 32/32 bibliography). Added to author-date production table.

**Quality gate:** 147 styles at fidelity 1.0 (up from 146).

**Branch:** style-evolve-pre-release-wave
