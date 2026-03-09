# Specifications

Feature and design specifications for non-trivial Citum capabilities.
A spec captures the *what* and *why* before implementation begins.

## Spec Template

Copy this template when creating a new spec:

```markdown
# [Feature Name] Specification

**Status:** Draft | Active | Superseded
**Version:** 1.0
**Date:** YYYY-MM-DD
**Supersedes:** (path, if any)
**Related:** (policy, bean, or issue)

## Purpose
One paragraph: what feature this specifies and why.

## Scope
In scope. Explicitly out of scope.

## Design
(Core content — decisions, data models, examples.)

## Implementation Notes
(Non-normative hints, known constraints.)

## Acceptance Criteria
- [ ] Verifiable condition 1
- [ ] Verifiable condition 2

## Changelog
- v1.0 (DATE): Initial version.
```

## Workflow

1. Create `docs/specs/FEATURE_NAME.md` (Status: `Draft`) **before** writing
   implementation code.
2. Commit the spec. Get it merged.
3. Set Status to `Active` in the same commit as the first implementation.
4. Reference the spec path in the bean description.

## Active Specs

| File | Feature |
|------|---------|
| [`ANNOTATED_BIBLIOGRAPHY.md`](./ANNOTATED_BIBLIOGRAPHY.md) | Document-scoped annotation overlay for bibliography rendering |
