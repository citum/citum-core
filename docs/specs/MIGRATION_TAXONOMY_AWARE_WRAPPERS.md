# Migration Taxonomy-Aware Wrappers Specification

**Status:** Draft
**Date:** 2026-04-23
**Related:** `docs/specs/STYLE_TAXONOMY.md`, `docs/specs/MIGRATE_RESEARCH_RICH_INPUTS.md`, `docs/policies/STYLE_WORKFLOW_DECISION_RULES.md`

## Purpose

Define how `citum-migrate` and `migrate-research` use the current Citum style
taxonomy during migration work. Migration should derive semantic class from the
registry, derive implementation form from the current checked-in style shape,
and emit wrapper output only when repo truth already establishes the parent
relationship.

## Scope

In scope:
- migration-time lineage resolution for a target CSL file
- distinction between semantic class and implementation form during migration
- automatic `extends:` emission for established profile and journal wrappers
- profile-contract guardrails for migration and `migrate-research`

Out of scope:
- new registry/schema fields for implementation form
- alias generation or alias discovery
- similarity-based parent inference for new wrappers

## Design

### Migration-Time Lineage

For a migration target, derive:

- `style_id` from the CSL filename stem
- semantic class from `StyleRegistry::resolve(style_id).kind` when the style has
  a canonical registry entry
- alias-backed journal classification when `style_id` resolves only through a
  parent entry's alias list
- current style shape from the checked-in Citum style with the same `style_id`,
  or from the embedded builtin style when the target is embedded

Implementation form is derived locally:

- `alias`: no concrete style file exists and the target resolves only as a registry alias
- `config-wrapper`: current style has `extends:` and no local template-bearing fields
- `structural-wrapper`: current style has `extends:` and local template-bearing fields
- `standalone`: current style has no `extends:`

### Automatic Wrapper Output

`citum-migrate` starts from the migrated standalone style, then may rewrite the
output into wrapper form.

Allowed cases:

- `profile + config-wrapper`
- `journal + config-wrapper`
- `journal + structural-wrapper`

For these cases, the parent comes only from the current checked-in style's
`extends:` value.

Fallback rule:

- if the current checked-in style does not establish a parent safely, keep
  standalone output

### Wrapper Emission Rules

For `config-wrapper` output:

- set `extends:` to the established parent
- keep only deltas relative to the resolved parent
- omit local template-bearing fields

For `structural-wrapper` output:

- set `extends:` to the established parent
- keep structural and non-structural deltas relative to the resolved parent

### Research Routing

`migrate-research` must report:

- target style or cluster
- semantic class
- implementation form
- selected parent, if any
- issue classification
- before/after evidence
- exact change made
- continue / stop / escalate decision

For `profile + config-wrapper` targets:

- migration work must preserve the config-wrapper contract
- if a proposed change requires local templates or local `type-variants`, the
  pass must reclassify or escalate instead of breaking the profile contract

For `journal + structural-wrapper` targets:

- structural-wrapper is a valid endpoint and must not be force-reduced to a thin wrapper

## Implementation Notes

- Semantic class remains `RegistryEntry.kind`; implementation form is derived in
  migration code and workflow logic.
- Low-level preset extraction may remain for option compression and family-like
  fixups, but it must not be treated as authoritative style identity.

## Acceptance Criteria

- [ ] `citum-migrate` derives semantic class from the registry and implementation form from current style structure.
- [ ] Known config-wrapper profiles and journals migrate with `extends:` and without local template-bearing fields.
- [ ] Known structural-wrapper journals preserve `extends:` plus structural deltas.
- [ ] Unknown or unresolved styles remain standalone.
- [ ] `migrate-research` output contracts require semantic class, implementation form, and parent reporting.

## Changelog

- 2026-04-23: Initial version.
