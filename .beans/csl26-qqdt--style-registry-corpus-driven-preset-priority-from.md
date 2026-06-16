---
# csl26-qqdt
title: 'style registry: corpus-driven preset priority from coverage-gap clusters'
status: todo
type: task
priority: normal
created_at: 2026-06-16T15:49:15Z
updated_at: 2026-06-16T15:49:15Z
---

The --coverage-gap preset-family clusters (csl26-t56t) provide corpus-wide data on which Citum base styles cover the most independent styles at Jaccard ≥ 0.65. Use this as the priority order for the preset/alias registry.

Top clusters by independent-style count:
1. elsevier-with-titles-core: 2698 styles
2. taylor-and-francis-cse-author-date[-core]: 998 styles
3. springer-basic-author-date[-core]: 969 styles
4. taylor-and-francis-chicago-author-date[-core]: 979 styles
5. apa-7th: 958 styles
6. chicago-author-date-18th: 952 styles
7. elsevier-with-titles: 790 styles
8. springer-basic-brackets[-core]: 776 styles
9. springer-vancouver-brackets: 774 styles
10. taylor-and-francis-national-library-of-medicine[-core]: 179 styles
11. ieee: 176 styles
12. chicago-notes-18th: 74 styles
13. modern-language-association: 67 styles

Observations:
- The -core / non-core pairs (e.g. springer-basic-author-date vs springer-basic-author-date-core) match nearly identical style sets, suggesting they differ only in minor rendering; consider whether both need registry entries or one can alias the other.
- elsevier-with-titles-core covers 2698 styles vs elsevier-with-titles's 790 — the core variant is the dominant one; document this.
- chicago-notes-18th only matches 74 styles but covers the humanities/notes niche which is strategically important.

Action: use these counts to order the preset priority list in docs/specs/STYLE_TAXONOMY.md and/or the style registry spec. Re-run --coverage-gap after converter fixes to see counts shift upward (currently depressed because many styles show false-positive gaps from the key-mapping bugs).
