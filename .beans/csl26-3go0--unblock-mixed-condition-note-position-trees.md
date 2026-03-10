---
# csl26-3go0
title: Unblock mixed-condition note position trees
status: todo
type: task
priority: normal
tags:
    - migration
    - styles
    - citations
created_at: 2026-03-10T22:20:45Z
updated_at: 2026-03-10T22:20:45Z
---

Follow-on from csl26-qfa3 and archive bean csl26-494i.

Problem:
XML-mode migration still falls back to the base citation template when note
styles mix position tests with other conditions in the same choose tree.

In scope:
- chicago-notes
- chicago-notes-bibliography-17th-edition
- mhra-notes
- mhra-notes-publisher-place
- mhra-notes-publisher-place-no-url
- new-harts-rules-notes
- new-harts-rules-notes-label-page
- new-harts-rules-notes-label-page-no-url

Deliverables:
- extend migrate support for mixed-condition note position trees without
  flattening sibling content
- add regression coverage for the blocked note-tree shapes
- re-run the note/legal batch from csl26-qfa3 after the migrate fix lands
- only open style-local cleanup follow-ons if specific styles still diverge
  after the shared migrate fix
