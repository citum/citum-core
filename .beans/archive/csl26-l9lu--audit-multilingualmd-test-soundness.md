---
# csl26-l9lu
title: Audit MULTILINGUAL.md test soundness
status: completed
type: task
priority: normal
created_at: 2026-06-12T14:11:15Z
updated_at: 2026-06-12T15:42:33Z
---

Run /test-soundness-review against docs/specs/MULTILINGUAL.md and crates/citum-engine/tests/multilingual.rs. Classify tests (good/suspicious/broken/redundant), review spec for ambiguity/contradiction/silence, fix/trim/add per skill, persist ledger row, open PR.

## Summary of Changes

Audited docs/specs/MULTILINGUAL.md against crates/citum-engine/tests/multilingual.rs (verdicts 4 good / 0 suspicious / 3 broken / 1 redundant).

- Fixed 3 broken short-contains tests with capture-and-pin exact assertions; the et-al test got a real kanji-author fixture (its old fixture had only Latin names).
- Deleted 1 redundant Arabic test (output byte-identical to its sibling) and its unused fixture item.
- Found the keyed-by-id JSON loader silently drops items that fail legacy CSL-JSON parsing — the fixture's three multilingual items were never loadable. Converted multilingual-cjk.json to native references format.
- Added a gap test for the §2.1 romanized-translated preset end-to-end via apa-7th.
- Filled the §3.1 Value Resolution placeholder; documented §3.4 locale-selected bibliography layouts (spec silences S1/S2).
- Upserted the MULTILINGUAL.md ledger row (addressed).
