---
# csl26-8br0
title: Common robust Chicago fixture (citation + bibliography, all variants)
status: todo
type: task
priority: high
created_at: 2026-06-30T14:29:49Z
updated_at: 2026-06-30T14:29:49Z
parent: csl26-40n4
blocked_by:
    - csl26-fr6f
---

Build one shared fixture with rich source types (book/chapter/periodical/media/archive/correspondence/recording/broadcast + original/event dates) exercising both citation and bibliography surfaces. Wire as benchmark_runs in scripts/report-data/verification-policy.yaml for all four Chicago variants. Add a bibliography surface to chicago-notes-18th (currently scopes: [citation] only) once the fixture exists. Replaces today's fragmented fixtures (references-expanded.json for author-date, references-humanities-note.json citation-only for notes, test-items-library/chicago-18th.json bibliography-only for author-date-18th).

## Todo
- [ ] Design shared fixture covering required reference types/scenarios for all 4 variants
- [ ] Build fixture JSON (or extend tests/fixtures/test-items-library/chicago-18th.json)
- [ ] Wire as benchmark_runs in verification-policy.yaml for all 4 variants
- [ ] Add bibliography scope to chicago-notes-18th policy entry
- [ ] Verify via report-core.js that all 4 variants report on the shared fixture
