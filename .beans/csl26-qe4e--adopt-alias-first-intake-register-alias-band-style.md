---
# csl26-qe4e
title: 'Adopt alias-first intake: register alias-band styles, do not migrate them'
status: todo
type: feature
priority: high
tags:
    - styles
    - taxonomy
    - migrate
created_at: 2026-07-17T18:00:12Z
updated_at: 2026-07-17T20:51:39Z
---

ACTION ITEM from the 2026-07-17 delta-derivability measurement. 1,674 of 2,844 independent CSL styles (59%) render >=0.98-similar to an already-registered style (scripts/report-data/alias-candidates-band-registered-2026-07-17.tsv). For these, the correct intake is a registry alias in registry/default.yaml, NOT migration: zero YAML to maintain, instant coverage. Do in human-reviewed batches (top-similarity first, per csl26-b4h2 guardrails: no false positives at >=0.98 in reviewed batches). This converts the measurement into coverage: registry entries ~165 -> potentially ~1,800+, and shrinks the future migration surface to the 1,011 near-clone band (delta candidates, csl26-10lt) plus true standalones. Related: csl26-8x90 (checked-in consolidation), csl26-zik7 (compat inheritance view). Context: docs/architecture/audits/2026-07-17_EXTENDS_DELTA_DERIVABILITY.md

## Review design (answer to: how do I human-review 1,674?)

You don't. Blended similarity >=0.98 is a SCREEN, not an acceptance bar (exact-match columns show 488/1,674 'alias-band' pairs match 0 fixture scenarios exactly). Tiered flow:

- Tier 0 - auto-accept (~112 today): citation AND bibliography exact-match rate 1.0/1.0 across all scenarios. Register with a 5% random spot-check, zero per-pair review.
- Tier 1 - family batches, scripted corroboration: high exact rates (e.g. >=0.95/0.95) plus CSL metadata corroboration (publisher/doc-link domain/title). One decision per parent family, not per pair - 8 targets cover ~1,145 candidates (elsevier-with-titles 633, american-fisheries-society 138, elsevier-harvard 78, ...). Human skims the batch list and reviews only flagged exceptions.
- Tier 2 - LLM pre-screen: a small-model agent renders side-by-side diffs for remaining pairs and emits one-line difference verdicts; human reads the one-liners, decides alias vs delta.
- Tier 3 - not aliases: low exact-match pairs (incl. the 488 at 0.0/0.0) route to the near-clone/delta path (csl26-10lt), never to aliasing.

Risk asymmetry: an alias is one registry line, trivially revertible, with a cheap escalation path (promote to delta wrapper on any reported divergence) - so the bar is 'no plausible difference flagged', not 'proven identical'.

Instrument note: consider re-keying the band column on exact-match rates rather than blended similarity (fold into csl26-10lt).
