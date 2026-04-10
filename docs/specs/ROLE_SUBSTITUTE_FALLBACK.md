# Role-Substitute Fallback Specification

**Status:** Active
**Date:** 2026-04-10
**Supersedes:** None
**Related:** `docs/specs/SECONDARY_CONTRIBUTOR_ROLE_FORMATTING.md`, `csl26-5ap9`, `csl26-mwnt`

## Purpose
Define the normative behavior of `options.substitute.role-substitute` so styles can declare role-aware contributor fallback chains without relying on engine-only assumptions or silently dropped custom roles.

## Scope
In scope: parsing and normalization of role names, fallback contributor resolution, suppression behavior for explicit fallback roles, locale-driven label rendering for substitute contributors, and documentation of custom-role handling. Out of scope: new schema keys, substitute preset redesign, or year-suffix/disambiguation behavior.

## Design
`options.substitute.role-substitute` is a map from a primary contributor role to an ordered list of fallback roles.

Both map keys and fallback entries normalize to canonical kebab-case contributor-role identifiers.

Built-in roles and custom roles are both valid. The engine must use one shared role-resolution path for:

- determining whether an explicit fallback contributor component should be suppressed because the primary role is present;
- resolving the fallback contributor data when the primary role is absent.

When a fallback role resolves to a contributor, the rendered substitute must follow the same name formatting and locale-driven role-label rules as other contributor rendering in the same context.

Custom roles remain valid even when they do not have a dedicated enum case. If a locale term exists for the custom role, the engine should use it for label rendering. If no locale term exists, the custom role still participates in contributor fallback and suppression; only the label text may be absent.

Unknown or unsupported role strings must not be silently dropped if they can be normalized and looked up as custom contributor roles.

## Implementation Notes
The initial implementation target is APA 7th chapter/container-author handling, but this behavior is generic and must not be hard-coded to APA-specific role names.

## Acceptance Criteria
- [x] `options.substitute.role-substitute` accepts built-in and custom contributor-role strings.
- [x] Role normalization is shared between fallback resolution and suppression checks.
- [x] Explicit fallback contributors are suppressed when the configured primary role is present.
- [x] Custom roles can participate in fallback without being silently ignored.
- [x] Substitute-path contributor labels use existing locale-driven role-label resolution when a locale term exists.

## Changelog
- 2026-04-10: Initial draft.
- 2026-04-10: Activated with shared role resolution, context-merge preservation, and APA closure coverage.
