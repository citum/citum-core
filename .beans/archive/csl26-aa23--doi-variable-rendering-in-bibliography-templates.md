---
# csl26-aa23
title: 'DOI output in bibliography via variable: doi'
status: completed
type: feature
priority: normal
created_at: 2026-03-16T11:48:22Z
updated_at: 2026-03-16T14:24:25Z
---

RSC and other styles need DOI output in bibliography. ITEM-1 (RSC oracle) expects DOI:10.1234/example but no mechanism exists. Unlocks RSC 33/33.

## Summary of Changes

Implemented a narrow `options.bibliography.article-journal.no-page-fallback: doi` policy so page-less journal bibliography entries can swap the standard detail block for DOI output without template conditionals.

Added engine behavior coverage, conservative migrate extraction for the legacy `if page ... else DOI` pattern, and updated `styles/royal-society-of-chemistry.yaml`. Targeted Rust checks passed, and `node scripts/oracle.js styles-legacy/royal-society-of-chemistry.csl --verbose` now reports bibliography 33/33.
