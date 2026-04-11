---
# csl26-twx1
title: 'engine: capitalize-first for verb-form role labels'
status: todo
type: bug
priority: high
created_at: 2026-04-11T11:34:00Z
updated_at: 2026-04-11T11:34:00Z
---

ContributorForm::Verb locale terms (e.g. 'edited by', 'translated by') are always lowercase. When the component appears sentence-initially (after a period separator), Chicago 18th and other styles expect 'Edited by'. The engine needs a capitalize-first mechanism for the verb-label path. Affects chicago-zotero-bibliography benchmark: 3+ items fail on this pattern.

Prerequisite design work is now captured in `docs/specs/SENTENCE_INITIAL_LABELS.md`. This bean remains open for the implementation PR that lands the rendering behavior after the broader sentence-initial label model is reviewed.
