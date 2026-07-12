---
# csl26-21d9
title: Configurable title-casing / sort-key stop words
status: todo
type: feature
priority: low
tags:
    - schema
    - sorting
    - locale
created_at: 2026-07-12T15:35:18Z
updated_at: 2026-07-12T16:02:05Z
parent: csl26-kcda
---

No mechanism found for a configurable stop-word list, needed for:
- title-case exemptions (a/an/the and similar) — CSL schema#106
- dropping articles when sorting titles — CSL schema#454
- Chicago 18th ed.'s new rule capitalizing prepositions of 5+ letters,
  which needs the same underlying word-list mechanism — CSL schema#456

Related but distinct: csl26-4kt3 (text-case token preservation for acronyms/
proper nouns in crates/citum-engine/src/values/text_case.rs) touches the same
code area but is a casing-transform correctness bug, not this configurability
feature — check for overlap/shared implementation surface before starting.

- [ ] Design: where does a stop-word list live (locale data vs style option)?
- [ ] Confirm relationship to csl26-4kt3's text_case.rs work
- [ ] Implement configurable stop-word list for title-casing and title-sorting
