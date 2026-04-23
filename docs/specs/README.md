# Specifications

Feature and design specifications for non-trivial Citum capabilities.
A spec captures the *what* and *why* before implementation begins.

## Spec Template

Copy this template when creating a new spec:

```markdown
# [Feature Name] Specification

**Status:** Draft | Active | Superseded
**Date:** YYYY-MM-DD
**Supersedes:** (optional path, if any)
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
- DATE: Initial version.
```

## Workflow

Before creating or converting a spec, consult
[`../guides/DOCUMENT_CLASSIFICATION.md`](../guides/DOCUMENT_CLASSIFICATION.md)
to make sure the document should be a spec rather than architecture or policy.

1. Create `docs/specs/FEATURE_NAME.md` (Status: `Draft`) **before** writing
   implementation code.
2. Commit the spec. Get it merged.
3. Set Status to `Active` in the same commit as the first implementation.
4. Reference the spec path in the bean description.

## Specs

| File | Feature |
|------|---------|
| [`ANNOTATED_BIBLIOGRAPHY.md`](./ANNOTATED_BIBLIOGRAPHY.md) | Document-scoped annotation overlay for bibliography rendering |
| [`GENERALIZED_RELATIONAL_CONTAINER_MODEL.md`](./GENERALIZED_RELATIONAL_CONTAINER_MODEL.md) | Recursive container model replacing flat CSL variables and Parent<T> |
| [`EDTF_ERA_LABEL_PROFILES.md`](./EDTF_ERA_LABEL_PROFILES.md) | Era label profiles and unspecified historical-year display |
| [`ARCHIVAL_UNPUBLISHED_SUPPORT.md`](./ARCHIVAL_UNPUBLISHED_SUPPORT.md) | ArchiveInfo/EprintInfo structs; Preprint type; archive_location semantic fix |
| [`EDTF_HISTORICAL_ERA_RENDERING.md`](./EDTF_HISTORICAL_ERA_RENDERING.md) | Locale-backed rendering of valid historical EDTF years |
| [`LOCALE_MESSAGES.md`](./LOCALE_MESSAGES.md) | ICU MF1 parameterized message system replacing flat YAML term files |
| [`ORIGINAL_PUBLICATION_RELATION_SUPPORT.md`](./ORIGINAL_PUBLICATION_RELATION_SUPPORT.md) | Universal original publication metadata support across all types |
| [`EMBEDDED_JS_TEMPLATE_INFERENCE.md`](./EMBEDDED_JS_TEMPLATE_INFERENCE.md) | Embedded `deno_core` live inference backend for `citum-migrate` |
| [`CONFIG_ONLY_PROFILE_OVERRIDES.md`](./CONFIG_ONLY_PROFILE_OVERRIDES.md) | Superseded profile-specific wrapper contract |
| [`UNIFIED_SCOPED_OPTIONS.md`](./UNIFIED_SCOPED_OPTIONS.md) | Breaking replacement for `options.profile` using normal scoped options |
| [`PROFILE_DOCUMENTARY_VERIFICATION.md`](./PROFILE_DOCUMENTARY_VERIFICATION.md) | Documentary-primary verification model for true profile wrappers |
