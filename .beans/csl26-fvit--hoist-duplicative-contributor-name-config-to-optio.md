---
# csl26-fvit
title: Hoist duplicative contributor name config to options defaults
status: todo
type: task
priority: normal
tags:
    - migrate
    - authorability
    - template
    - dx
created_at: 2026-06-16T12:55:29Z
updated_at: 2026-06-16T12:55:29Z
---

Migrated styles repeat 'form: long / name-order: family-first-only / and: text' on every contributor in every type-variant (see ACME). Should hoist common contributor render config to bibliography-level options.contributors and omit per-component where it matches the resolved default.

Care: citation (family-first) vs bibliography (family-first-only) differ; per-role config differs. Omit only on exact match against effective default.

Authorability follow-up from ACME review (PR #932). Pre-existing; not a regression.
