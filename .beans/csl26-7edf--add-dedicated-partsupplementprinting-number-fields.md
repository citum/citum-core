---
# csl26-7edf
title: Add dedicated part/supplement/printing number fields
status: todo
type: feature
priority: deferred
tags:
    - schema
    - engine
created_at: 2026-03-05T15:22:41Z
updated_at: 2026-04-25T20:20:06Z
parent: csl26-li63
---

Tentative conclusion from PR #285 analysis:

- We should add dedicated fields for `part-number`, `supplement-number`, and
  `printing-number` instead of mapping them to `reference.number()`.
- Current fallback to `reference.number()` is semantically incorrect because it
  can return unrelated numbers (report/session/docket/etc.).
- Interim behavior already applied in PR #285: these number variables resolve to
  `None` until dedicated fields exist.

Recommended implementation scope:

- Add optional fields to reference types where these values are relevant.
- Add conversion/ingestion mapping for supported sources (legacy CSL JSON,
  biblatex fields where available).
- Wire TemplateNumber resolution directly to the new fields and add focused
  tests for each variable.

Definition of done (tentative):

- `part-number`, `supplement-number`, and `printing-number` render from their
  own fields only.
- No regression in existing `number` variable behavior.
- Schema docs and examples include the new fields.
