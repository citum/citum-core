---
# csl26-txlm
title: 'Turabian: student title-page layout in StyleVariantDelta'
status: todo
type: feature
priority: normal
created_at: 2026-03-18T11:00:00Z
updated_at: 2026-03-18T11:00:00Z
blocked_by:
    - csl26-fsjy
---

Implement student title-page handling for the Turabian variant of the
Chicago Notes 18th preset. This is the remaining deviation from
`chicago-shortened-notes-bibliography.yaml` that could not be expressed via
existing schema fields in csl26-fsjy.

## Background

Turabian 9th edition (Kate L. Turabian, *A Manual for Writers*) adds a
student-paper title-page layout requirement on top of Chicago Notes 18th.
This is a presentation-layer concern (page layout, not citation formatting)
that falls outside the current `StyleVariantDelta` schema.

## Design intent from csl26-fsjy

`StyleVariantDelta.custom` was added explicitly as a forward-compatible escape
hatch for this use case. A Turabian style YAML can prototype the feature today:

```yaml
preset: chicago-notes-18th
variant:
  custom:
    student-paper:
      title-page: true
      institution: "University of Chicago"
```

The engine ignores `custom` at render time until this bean adds consuming logic.

## Tasks

- [ ] Design the `student-paper` sub-schema (title-page fields, institution, etc.)
- [ ] Add `student_paper: Option<StudentPaperConfig>` to `StyleVariantDelta`
      (graduating the prototype from `custom` to a typed field)
- [ ] Implement engine rendering for the student title-page block
- [ ] Add a `turabian-student.yaml` example style
- [ ] Update `STYLE_PRESET_ARCHITECTURE.md` §3 with the new field
- [ ] Update `citum.schema.json`
