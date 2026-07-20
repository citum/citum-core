---
# csl26-epu6
title: Add opaque date note (calendar annotation) support
status: in-progress
type: feature
priority: high
tags:
    - dates
    - gb-t
    - rendering
    - schema
created_at: 2026-07-20T14:00:23Z
updated_at: 2026-07-20T14:00:43Z
blocking:
    - csl26-0kqf
---

Implement the CALENDAR_DATE_ANNOTATIONS.md spec: DateValue{value, note} with scalar/mapping serde, DateConfig.note-wrap render opt-in (bibliography-scoped), script-aware wrap via realize_wrap, legacy GB/T note-field conversion, and GB/T style enablement. Completes PR 1068 (codex/calendar-date-annotations).

## Todo

- [x] Step A: revise CALENDAR_DATE_ANNOTATIONS.md, DATE_MODEL.md, GBT_7714_CITATION_CONVENTIONS.md, docs/specs/README.md for the `note` rename and `DateConfig.note-wrap` config; squash onto PR 1068 branch
- [ ] Step B: `DateValue{value, note}` scalar/mapping serde in citum-schema-data
- [ ] Step B: route date accessors through `DateValue.value`
- [ ] Step B: `DateConfig.note_wrap: Option<WrapConfig>` in citum-schema-style
- [ ] Step B: bibliography-scoped render via `realize_wrap`
- [ ] Step B: legacy full-width-paren conversion in citum-migrate
- [ ] Step B: enable `note-wrap: parentheses` in GB/T style bibliography options
- [ ] Step B: flip spec Status Draft -> Active in the implementation commit
- [ ] Step B: `just schema-gen`; commit regenerated schemas
- [ ] Step B: tests (serde, processing invariance, render, scope, output-format escaping, migrate)
- [ ] Verify: `just pre-commit`, GB/T workflow-test/oracle, report-core fidelity check
- [ ] Push PR 1068, `gh pr checks 1068 --watch`
