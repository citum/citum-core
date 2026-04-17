---
# csl26-1nsh
title: chicago-author-date-18th translator+patent
status: completed
type: task
priority: high
created_at: 2026-04-17T00:20:10Z
updated_at: 2026-04-17T11:22:57Z
parent: csl26-12hl
---

Translator rendering + patent format. Zotero benchmark at 73.3%.

## Summary of Changes

Engine fixes landed on feat/style-co-evolution-wave-2026-04:

- **Sentence-initial capitalization**: removed early-return guard in
  `apply_bibliography_sentence_initial_context` that blocked verb-form
  contributor labels ("edited by" → "Edited by") for entries without
  explicit prefixes; added value-path capitalize for the no-prefix case
- **display-as-sort: first**: scoped family-first ordering to the first
  contributor only; subsequent contributors now render given-first as spec
- **Title-case improvements**: capitalize after ? and !; add "from" to
  stop words; hyphenated compound handling (Eighteenth-Century);
  punctuation stripping before stop-word check ((and → treated as and)

fidelity: 0.695 → 0.794 (365/471 bib passing); Zotero benchmark 73.8% (≥73% gate)

Remaining 106 failures require complex engine features (original-date
formatting, uncertain dates [1772?], translator in articles, media/audio
formats, nested volumes) — deferred to a future wave.
