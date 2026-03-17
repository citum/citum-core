---
# csl26-4mzf
title: Rename style files to short-name + edition convention
status: todo
type: task
priority: normal
created_at: 2026-03-17T11:11:49Z
updated_at: 2026-03-17T11:12:12Z
blocked_by:
    - csl26-fsjy
---

Rename well-known styles to {short-name-kebab}[-{edition-kebab}].yaml.

## Renames required

| Current | New |
|---------|-----|
| modern-language-association.yaml | mla-9th.yaml |
| chicago-notes.yaml | chicago-notes-18th-edition.yaml |
| chicago-notes-bibliography-17th-edition.yaml | chicago-notes-17th-edition.yaml |
| chicago-shortened-notes-bibliography.yaml | chicago-shortened-notes-18th-edition.yaml |
| chicago-author-date.yaml | chicago-author-date-18th-edition.yaml |
| mhra-notes.yaml | mhra-4th-edition.yaml |
| oscola.yaml | oscola-4th-edition.yaml |
| harvard-cite-them-right.yaml | harvard.yaml |
| american-medical-association.yaml | ama.yaml |
| ieee.yaml | (already correct) |
| apa-7th.yaml | (already correct) |
| nature.yaml | (already correct) |
| elsevier-harvard.yaml | (already correct — publisher identity) |
| elsevier-vancouver.yaml | (already correct — publisher identity) |

## Scope

- Update all filenames in `styles/`
- Update any references in `scripts/`, `tests/`, `docs/`, `CLAUDE.md`
- Journal/publisher styles (Springer, Elsevier, NLM variants, etc.) are NOT renamed
- Confirm preset key shape matches filename slug before executing (csl26-fsjy)

## Notes

The `-notes` / `-author-date` suffixes in Chicago filenames are retained for
now to distinguish format variants. Once multi-format preset support exists,
revisit whether a single `chicago-18th-edition.yaml` with internal variant
selection is preferable.
