---
# csl26-fr6f
title: 'Chicago family audit: shared vs ordering vs missing facts'
status: completed
type: task
priority: high
created_at: 2026-06-30T14:29:40Z
updated_at: 2026-06-30T14:34:19Z
parent: csl26-40n4
---

Compare type-variants/type-templates across the four embedded Chicago YAMLs (chicago-author-date-18th, chicago-notes-18th, chicago-shortened-notes-bibliography, taylor-and-francis-chicago-author-date) per source type. Classify each rule as shared component, order-layer (style-specific), or missing conversion/accessor fact. Output: docs/architecture/audits/2026-06-30_CHICAGO_FAMILY_AUDIT.md.

## Todo
- [x] Per-source-type comparison table across all four YAMLs
- [x] Classification: shared component vs order-layer vs missing fact
- [x] List of facts absent from Citum conversion/accessors (archival correspondence, recordings, performances, broadcasts, original-pub dates, event dates, note-derived roles), tagged to variant(s) unblocked
- [x] Recommendation on where a hidden common base is safe vs where it would force a wrong rendered order

## Summary of Changes

Produced docs/architecture/audits/2026-06-30_CHICAGO_FAMILY_AUDIT.md covering all four embedded Chicago variants (chicago-author-date-18th, chicago-notes-18th, chicago-shortened-notes-bibliography(-core), taylor-and-francis-chicago-author-date(-core)).

Key findings:
- Confirmed no shared base exists today (author-date-18th extends book, notes-18th extends dataset).
- Section A: 7 genuinely shared-component candidates safe for a hidden chicago-18-base (page-range-format, punctuation-in-quote, demote-non-dropping-particle, multilingual, DOI prefix convention, shared message patterns, personal-communication bib suppression policy).
- Section B: date-position-relative-to-author and citation grammar confirmed as true order-layer differences that must stay separate.
- Section C: 6 missing conversion/accessor facts identified and prioritized (archival correspondence fields, recordings/broadcasts typed accessor, original publication dates, event dates, note-derived roles, patent/issued-date shared message pattern).
- Flagged that T&F currently silently inherits CMOS (not Style F) rendering for 7 source types it doesn't override — noted for the final-tuning child bean.

Findings feed directly into csl26-8br0 (fixture), csl26-zs0f (component base), csl26-ifhx (conversion facts), and csl26-h7oc (final tuning).
