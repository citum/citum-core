---
# csl26-mlt4
title: Extract CSL-M multilingual test data
status: deferred
type: task
priority: low
created_at: 2026-02-12T00:00:00Z
updated_at: 2026-03-01T17:00:00Z
---

Extract CJK/Arabic/Russian test cases from Juris-M/jm-styles repository for multilingual processor validation.

Focus on styles without legal extensions. Store in tests/fixtures/multilingual/

## Status

**Deferred**: Multilingual fixture extraction requires detailed audit of CSL test suite structure and manual conversion of proprietary CSL test syntax to Citum JSON. This is preparatory work for multilingual name rendering tests (mlt1/mlt2) but is not critical for the current sprint. Placed on backlog for future multilingual validator expansion.

The `tests/fixtures/multilingual/` directory already contains:
- `multilingual-cjk.json` - Existing CJK fixtures (pre-seeded)
- `multilingual-cyrillic.json` - Existing Cyrillic fixtures (pre-seeded)
- `multilingual-mixed.json` - Existing mixed-script fixtures (pre-seeded)

Full CSL-M test suite ingestion deferred pending multilingual processor roadmap prioritization.

Refs: csl26-mlt1, csl26-mlt2
