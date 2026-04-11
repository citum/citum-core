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
