---
# csl26-7grx
title: Expand citations fixture for alias-discovery blind spots
status: todo
type: task
created_at: 2026-04-19T13:11:00Z
updated_at: 2026-04-19T13:11:00Z
---

Add 4 missing scenario types to tests/fixtures/citations-expanded.json so find-alias-candidates.js can distinguish same-family variants: (1) subsequent/repeated cite, (2) note-context citation, (3) bracket/delimiter shape variation, (4) archive reference with both archive and archive-place. Unblocks promoting 8 withheld 1.000-score candidates (AMA brackets/parens, chicago-shortened-* variants, MLA notes) to registry aliases. See docs/guides/ALIAS_DISCOVERY.md#known-fixture-blind-spots.
