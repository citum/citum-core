---
# csl26-7grx
title: Expand citations fixture for alias-discovery blind spots
status: completed
type: task
priority: normal
tags:
    - testing
created_at: 2026-04-19T13:11:00Z
updated_at: 2026-06-02T10:49:35Z
---

Add 4 missing scenario types to tests/fixtures/citations-expanded.json so find-alias-candidates.js can distinguish same-family variants: (1) subsequent/repeated cite, (2) note-context citation, (3) bracket/delimiter shape variation, (4) archive reference with both archive and archive-place. Unblocks promoting 8 withheld 1.000-score candidates (AMA brackets/parens, chicago-shortened-* variants, MLA notes) to registry aliases. See docs/guides/ALIAS_DISCOVERY.md#known-fixture-blind-spots.

## Summary of Changes

Work completed in 1349b3c0. Added subsequent-same-item and archive-single to EXTRA_SCENARIOS in scripts/find-alias-candidates.js (inline, not shared fixture, to avoid oracle snapshot invalidation). Fixed citation_match to exact string equality for bracket/delimiter detection. Promoted modern-language-association-notes to confirmed alias. All 4 blind spots resolved and documented in docs/guides/ALIAS_DISCOVERY.md.
