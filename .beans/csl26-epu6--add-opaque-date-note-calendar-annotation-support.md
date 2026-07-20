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
updated_at: 2026-07-20T15:22:44Z
blocking:
    - csl26-0kqf
---

Implement the CALENDAR_DATE_ANNOTATIONS.md spec: DateValue{value, note} with scalar/mapping serde, DateConfig.note-wrap render opt-in (bibliography-scoped), script-aware wrap via realize_wrap, legacy GB/T note-field conversion, and GB/T style enablement. Completes PR 1068 (codex/calendar-date-annotations).

## Todo

- [x] Step A: revise CALENDAR_DATE_ANNOTATIONS.md, DATE_MODEL.md, GBT_7714_CITATION_CONVENTIONS.md, docs/specs/README.md for the `note` rename and `DateConfig.note-wrap` config; squash onto PR 1068 branch
- [x] Step B: extend the existing date type (`EdtfString` -> `DateValue`) in place with scalar/mapping serde, in citum-schema-data (no new wrapper type)
- [x] Step B: route date accessors through `DateValue.value`
- [x] Step B: `DateConfig.note_wrap: Option<WrapConfig>` in citum-schema-style
- [x] Step B: bibliography-scoped render via `realize_wrap` (`append_note` in citum-engine/src/values/date.rs)
- [x] Step B: legacy full-width-paren conversion (`annotated_issued_from_legacy`, citum-schema-data/src/reference/conversion/mod.rs — not citum-migrate; that crate migrates CSL styles, not reference data)
- [x] Step B: enable `note-wrap: parentheses` in gb-t-7714-2025-author-date.yaml bibliography.options.dates
- [x] Step B: flip spec Status Draft -> Active in the implementation commit
- [x] Step B: `just schema-gen`; commit regenerated schemas
- [x] Step B: tests added (serde round-trip/unknown-field, processing-invariance/collision, render single+interval+year-suffix+script, bibliography-vs-citation scoping, HTML escaping, legacy conversion incl. half-width disjointness). Not yet covered: Djot/Markdown/LaTeX/Typst/org escaping specifically (HTML is the highest-risk format and is covered; the others share the same fmt.text() escaping path).
- [x] Verify: `just pre-commit` green (fmt/clippy/full nextest, 2093+ tests). Not run: GB/T workflow-test/oracle corpus comparison, report-core fidelity report (heavy full-corpus checks; change is additive/backward-compatible and covered by unit+integration tests instead).
- [ ] Push PR 1068, `gh pr checks 1068 --watch`
