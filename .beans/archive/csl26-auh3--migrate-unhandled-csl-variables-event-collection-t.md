---
# csl26-auh3
title: 'migrate: audit and fix stale coverage-gap variable claims'
status: completed
type: feature
priority: normal
tags:
    - migrate
    - coverage-gap
created_at: 2026-06-16T15:49:01Z
updated_at: 2026-06-16T20:15:55Z
---

The original bean was created from `citum-analyze --coverage-gap` output and
overstated several Citum-side gaps. Current code evidence:

- `names:translator` is already mapped by `citum-migrate` to
  `ContributorRole::Translator`, and `Reference::translator()` exists.
- `date:original-date` is already mapped to `DateVariable::OriginalPublished`
  and rendered from the `original` relation via `Reference::original_date()`.
- `var:authority` is already mapped to `SimpleVariable::Authority`.
- `var:year-suffix` remains processor-generated disambiguation state, not a
  template variable.

Real gaps fixed in the csl26-auh3 PR:

- `citum-analyze` now reverse-maps `DateVariable::OriginalPublished` to CSL
  `date:original-date` instead of reporting a stale analyzer gap.
- `var:section` now compiles to `SimpleVariable::Section`.
- CSL `collection-title` is treated as a series/collection title, not as
  `container-title`. Migration now emits `TitleType::CollectionTitle`, CSL-JSON
  conversion preserves it as a nested parent-series relation, and the engine can
  render it through `Reference::collection_title()`.

Follow-up:

- `var:event` is still a render-surface gap. Citum already has an `Event` data
  model and promoted conference-event relations, so this is not a missing data
  model. It needs a separate template/accessor design that renders CSL `event`
  from `Event.title` and/or promoted event relations without conflating it with
  `container-title` or `collection-title`.
