# Unified Scoped Options Specification

**Status:** Active
**Date:** 2026-04-22
**Supersedes:** `CONFIG_ONLY_PROFILE_OVERRIDES.md`
**Related:** `STYLE_TAXONOMY.md`, `STYLE_PRESET_ARCHITECTURE.md`, bean `csl26-xt7k`, bean `csl26-nrkn`, bean `csl26-rwgi`

## Purpose

This specification replaces the author-facing `options.profile` contract with
normal typed options that live at the scope they actually affect. Profile
styles remain a registry/taxonomy concept, but profile-specific configuration
does not. Authors configure wrappers and standalone styles with the same
schema surface.

## Scope

In scope:

- removal of `options.profile` from the public schema
- new citation-scoped and bibliography-scoped option fields for inherited and
  standalone styles
- resolver changes required to apply these fields during normal style
  resolution
- migration of embedded profile wrappers and documentation

Out of scope:

- stringly typed variable/parameter systems
- preserving compatibility with `options.profile`
- widening inheritance so profile wrappers can override template-bearing fields

## Design

### 1. Author-Facing Model

The schema no longer exposes a dedicated profile-only namespace.

- profile styles still use `extends:` to select a structural base
- profile styles may still not override template-bearing fields
- style configuration uses normal typed options in the scope they affect

The initial replacement fields are:

- top-level `options.contributors`
- `citation.options.label-wrap`
- `citation.options.group-delimiter`
- `bibliography.options.label-mode`
- `bibliography.options.label-wrap`
- `bibliography.options.date-position`
- `bibliography.options.title-terminator`
- `bibliography.options.repeated-author-rendering`
- existing `bibliography.options.volume-pages-delimiter`

### 2. Resolution Model

Style resolution keeps the current structural rule for profile wrappers:

- profile wrappers inherit templates intact from their base
- profile wrappers may not override template-bearing fields

After a style is resolved, the engine applies the new scoped options to the
effective citation and bibliography specs. This happens for both profile
wrappers and standalone styles, so the option semantics are uniform.

### 3. Schema Rules

`options.profile` is removed completely.

- parsing a style with `options.profile` is a hard error
- the error message must point authors to the new scoped fields
- capability-gated profile-axis validation is removed

## Implementation Notes

The new fields remain strongly typed Rust schema items. This preserves the
current code-as-schema model while removing the separate profile vocabulary.

## Acceptance Criteria

- [ ] Styles using the new scoped fields parse and resolve.
- [ ] Styles using `options.profile` fail with a migration-oriented error.
- [ ] Embedded profile wrappers use only the new scoped fields.
- [ ] Standalone styles can use the same fields without `extends:`.

## Changelog

- 2026-04-22: Activated alongside the schema and embedded-style migration.
- 2026-04-22: Initial version.
