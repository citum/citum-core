---
# csl26-8eom
title: Split citation and bibliography option schemas
status: todo
type: feature
priority: normal
created_at: 2026-03-25T20:45:08Z
updated_at: 2026-03-25T20:45:08Z
---

Split the shared `Config` surface used by `CitationSpec.options` and
`BibliographySpec.options`.

- Define citation-only and bibliography-only option types.
- Decide compatibility strategy for current YAML.
- Update schema/docs so bibliography-only fields cannot be silently ignored in
  citation contexts.
