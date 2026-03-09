# Policies

Active behavioral rules that agents and contributors must follow.
Each policy states a binding rule in its opening **Rule** section —
readable without scrolling further.

## Policy Template

Copy this template when creating a new policy:

```markdown
# [Policy Name]

**Status:** Active | Superseded
**Version:** 1.0
**Date:** YYYY-MM-DD
**Superseded by:** (path, if Superseded)
**Related:** (spec or CLAUDE.md section)

## Rule
The binding rule in 1–2 sentences. Agents must be able to apply this
without reading further.

## Rationale
Why this rule exists.

## Application
Decision table or flowchart for common scenarios.

## Exceptions
When the rule may be bypassed, and the approval process.

## Changelog
- v1.0 (DATE): Established.
```

## Active Policies

| File | Rule summary |
|------|-------------|
| [`TYPE_ADDITION_POLICY.md`](./TYPE_ADDITION_POLICY.md) | How to add new reference types to the schema |
| [`SQI_REFINEMENT_PLAN.md`](./SQI_REFINEMENT_PLAN.md) | Current SQI scoring refinement direction |
