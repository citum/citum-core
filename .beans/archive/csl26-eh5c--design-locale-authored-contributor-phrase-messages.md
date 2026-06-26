---
# csl26-eh5c
title: Design locale-authored contributor phrase messages
status: completed
type: feature
priority: high
tags:
    - localization
    - mf2
    - contributors
    - styles
created_at: 2026-06-26T12:29:18Z
updated_at: 2026-06-26T12:45:14Z
---

## Problem

PR #966 migrated checked-in style phrase glue away from rendered template
`term:` components and into locale-owned `message: pattern.*` calls. It
deliberately did not solve contributor-plus-role phrase realization.

AMA-style `In:` editor/title phrasing and APA-style container
contributor/title phrasing still need a locale-owned message model. The locale
needs to control how rendered names, role labels, counts, genders, punctuation,
and rendered container/title fragments are ordered around each other without
forcing English glue back into style templates.

## Scope

Design the contributor phrase message model before implementation. This bean
owns the argument shape, initial motivating cases, and acceptance criteria for a
later implementation PR.

Do not treat contributor `label.term` as deprecated template `term:`.
`label.term` remains the schema-native lexical role-label mechanism until and
unless the contributor phrase model deliberately replaces a complete
role-plus-name phrase.

## Acceptance Criteria

- Define the argument model for contributor phrase messages: rendered names,
  role label/form, contributor count/gender where needed, and rendered
  container/title fragments.
- Decide initial `pattern.*` IDs in the design/implementation PR, not in this
  tracking bean.
- Cover motivating cases from AMA `In:` editor/title phrasing and APA
  container-contributor/title phrasing.
- Preserve `label.term` for lexical role labels; do not treat it as deprecated template `term:`.
- Add tests or fidelity checks in the eventual implementation PR for AMA, APA,
  and at least one reordered locale example.

## Related

- Split from `csl26-fdzc` after PR #966 completed the checked-in rendered
  template `term:` migration.
- Spec context: `docs/specs/LOCALE_MESSAGES.md`.


## Completion

Specified in `docs/specs/CONTRIBUTOR_PHRASE_MESSAGES.md`, with cross-links from `docs/specs/LOCALE_MESSAGES.md` and `docs/specs/README.md`. The spec defines the initial contributor phrase message IDs, argument contracts, rationale for why term-based and role-label-only approaches are insufficient for diverse locales, and acceptance criteria for the later implementation PR.
