# Secondary Contributor Role Formatting Specification

**Status:** Active
**Version:** 1.0
**Date:** 2026-03-22
**Supersedes:** None
**Related:** `docs/architecture/NAME_FORMATTING.md`, `docs/specs/LOCALE_MESSAGES.md`

## Purpose
Define a coherent configuration and verification model for secondary contributor roles so Citum can render editors, translators, interviewers, recipients, and related role-bearing contributors consistently across citation and bibliography contexts.

## Scope
In scope: role-label presets, legacy `editor-label-format` compatibility, substitute-path parity, fixture coverage, and oracle diagnostics for secondary contributor rendering. Out of scope: new reference-data model fields for currently unsupported contributor relations and any redesign of the existing contributor or substitute preset systems.

## Design
Secondary contributor role formatting is additive and non-breaking. Styles may configure a global role-label preset at `options.contributors.role.preset` and override it per role at `options.contributors.role.roles.<role>.preset`.

**When to use each form:**

Use `role: { preset: <value> }` when the same preset applies to all roles — this is the preferred concise form:

```yaml
contributors:
  role:
    preset: short-suffix
```

Use `role.roles` only when roles diverge (different presets) or when a role needs non-preset overrides (`name-order`, `emph`, `prefix`, `suffix`):

```yaml
contributors:
  role:
    preset: short-suffix        # default for all roles
    roles:
      editor:
        preset: long-suffix     # editors differ
      translator:
        emph: true              # translators get an extra override
```

Repeating the same preset for every role in `roles` is redundant and must be collapsed to the global form.

This wave defines the following presets:

- `none`: suppress the configured role label.
- `verb-prefix`: render a localized verb form before the contributor names.
- `verb-short-prefix`: render a localized short verb form before the contributor names.
- `short-suffix`: render a localized short label after the contributor names.
- `long-suffix`: render a localized long label after the contributor names.

Preset precedence is:

1. Explicit `TemplateContributor.label`
2. Per-role preset override
3. Global role preset
4. Legacy `editor-label-format` compatibility for editor and translator roles
5. Existing form-based defaults

Role-aware substitution must use the same locale-driven label resolution as normal contributor rendering. When `options.substitute.contributor-role-form` is present, it overrides the configured role-label preset for substitute labels only; otherwise substitute rendering falls back to the same role preset resolution path as ordinary role-bearing contributors.

Verification must include a dedicated secondary-role fixture set that is run in addition to each style family’s existing sufficiency fixtures. Oracle diagnostics should expose secondary-role component buckets so role-heavy bibliography mismatches are inspectable beyond full-string diffs, and contributor component extraction must preserve the rendered name span so name-order and initialization regressions remain visible.

## Implementation Notes
The current data model does not expose dedicated `collection-editor` or `container-author` accessors. This specification still treats them as first-class preset targets and locale terms, but this wave verifies role formatting primarily through currently rendered secondary roles: editor, translator, interviewer, and recipient, plus container-editor behavior represented through chapter parent editors.

## Acceptance Criteria
- [ ] Styles can configure global and per-role secondary contributor label presets without breaking existing YAML.
- [ ] `editor-label-format` is removed; styles use `options.contributors.role.preset` or per-role `role.roles` overrides.
- [ ] Translator substitution uses locale-aware role formatting instead of a hardcoded label.
- [ ] Secondary-role fixture runs are included in structured oracle planning for the main style families.
- [ ] Oracle diagnostics expose secondary-role component buckets for bibliography mismatches.
- [ ] New secondary-role integration coverage follows the repo BDD rule: only parameterized multi-case integration tests use `given_..._when_..._then_...` naming.

## Changelog
- v1.0 (2026-03-22): Initial version.
