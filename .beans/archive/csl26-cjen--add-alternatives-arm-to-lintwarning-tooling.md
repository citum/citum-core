---
# csl26-cjen
title: Add Alternatives arm to lint/warning tooling
status: scrapped
type: task
priority: low
tags:
    - engine
    - schema
    - style
created_at: 2026-09-07T12:05:24Z
updated_at: 2026-09-08T12:47:08Z
blocked_by:
    - csl26-57a7
---

Once alternatives: ships (docs/specs/ALTERNATIVES.md), two tooling functions still special-case Group but not Alternatives, so a candidate's content is invisible to them: lint.rs's collect_template_requirements (locale-requirement collection for style linting) and api/warnings.rs's scan_template_for_unknowns (unknown-enum-variant warnings). Neither affects rendered output -- found during csl26-57a7's consumer audit, split out because it's tooling completeness, not a rendering-correctness gate on alternatives:'s own Acceptance Criteria.

## Reasons for Scrapping

Same reason as csl26-57a7: this bean assumed a new `TemplateComponent::Alternatives` variant that lint.rs and api/warnings.rs would need their own arm for. The redesigned `select: first` (docs/specs/GROUP_SELECT.md) is a mode on the existing `TemplateComponent::Group`, which both tools already handle generically. No arm needed, nothing to do here.
