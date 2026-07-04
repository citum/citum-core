---
# csl26-boql
title: Note punctuation rules to locale data; dedupe notes.rs
status: todo
type: task
tags:
    - notes
    - localization
parent: csl26-8m2p
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-04T17:49:02Z
---

locale_note_rule hardcodes en-US/fr punctuation-placement defaults in engine code rather than locale files (overridable via options.notes but defaults belong in data), and process_note_document/_html plus prepare_note_citations/_html are ~70-line near-verbatim duplicates with silent render-error fallbacks. Move defaults to locale data and extract the shared note pipeline. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 21.
