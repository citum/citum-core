---
# csl26-fqug
title: 'New reference types: indigenous knowledge, sacred works'
status: todo
type: feature
priority: low
tags:
    - taxonomy
    - schema
created_at: 2026-07-12T15:35:48Z
updated_at: 2026-07-12T16:02:13Z
parent: csl26-kcda
---

Two reference-type gaps needing citation-form design work, not mechanical
type-taxonomy additions:

- Indigenous/traditional-knowledge sources (Chicago 18th ed. protocols:
  nation/community, treaty territory, topic of communication) — CSL schema#446
- Sacred-work type for Bible/Quran-style citation (no added punctuation,
  space-joined locator, no bibliography entry) — CSL schema#447. Note:
  `classic` exists only as a locale term in Citum today, not a backing
  MonographType/genre value — the "classic" strategy #447 points to as
  prior art doesn't actually exist yet either.

- [ ] Design citation-form requirements with domain/style-guide research
      (this is not a simple enum addition — both need their own rendering
      rules, see DOMAIN_EXPERT.md workflow)
- [ ] Consider whether `classic` should become a real type in the same pass
