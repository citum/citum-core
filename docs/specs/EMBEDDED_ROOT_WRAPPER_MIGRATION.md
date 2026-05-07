# Embedded Root Wrapper Migration Specification

**Status:** Active
**Date:** 2026-05-07
**Related:** `STYLE_TAXONOMY.md`, `MIGRATION_TAXONOMY_AWARE_WRAPPERS.md`, `STYLE_PRESET_ARCHITECTURE.md`, `../architecture/2026-05-07_SQI_INTEGRITY_AUDIT.md`

## Purpose

Define how migration and upgrade workflows produce a corpus made of embedded
style roots plus thin public wrappers. The workflow must preserve fidelity and
authority. It must not infer parentage from output similarity alone.

## Scope

In scope:

- explicit migration output plans
- embedded-root plus public-wrapper artifact production
- proof gates for wrapper emission
- SQI interaction with root `extends:` and diff-form `type-variants`

Out of scope:

- automatic alias discovery
- similarity-only parent selection
- changing schema merge semantics for arrays
- broad corpus rewrites without a bounded, verified wave

## Design

Migration output is classified before writing artifacts:

| Plan | Artifacts | Use |
|---|---:|---|
| `Standalone` | 1 | no safe parent/root relationship is established |
| `ExistingWrapper` | 1 | repo truth already establishes `extends:` parentage |
| `CreateEmbeddedRootAndWrapper` | 2 | workflow explicitly creates a hidden root and public wrapper |
| `UpgradeEmbeddedRootAndWrapper` | 2 | workflow updates an existing hidden root and its public wrapper |

Parent/root selection must come from one of:

1. current publisher or journal guidance
2. current publisher house rules or submission instructions
3. named parent manual or standards reference
4. existing repo registry, embedded root, or checked-in wrapper truth

CSL XML structure and output similarity may support verification, but they are
not sufficient authority for selecting a parent.

### Wrapper Rules

Config wrappers:

- must set top-level `extends:`
- may keep local metadata and scoped options
- must not keep local templates, local `type-variants`, or template-clearing
  `null` values

Structural wrappers:

- must set top-level `extends:`
- may keep schema-backed structural deltas
- should prefer Template V3 diff-form `type-variants` over copied full variants
  when the diff resolves mechanically

Embedded-root plans:

- are opt-in multi-artifact writes
- create or update the hidden root under `styles/embedded/`
- keep the public style as a wrapper over that root
- must update embedded style loading and registry-facing metadata consistently

### Proof Gates

Before accepting root/wrapper output, the workflow must:

1. generate the intended standalone or effective style
2. generate the root plus wrapper artifacts
3. resolve the wrapper through normal schema resolution
4. compare the resolved wrapper against the intended effective style on
   schema-relevant fields
5. run the style oracle or the bounded wave report
6. reject the wrapper if fidelity regresses or if the profile contract is broken

If proof fails, the workflow keeps `Standalone` or `StructuralWrapper` output and
records the infrastructure constraint.

## Implementation Notes

`citum-migrate` exposes `MigrationOutputPlan` from lineage analysis so callers
can distinguish single-artifact existing wrappers from explicit multi-artifact
embedded-root work. The default CLI remains single-artifact and does not create
hidden roots unless a future workflow-facing command opts into that plan.

SQI reports must score authored wrapper shape, not inherited parent complexity.
Root `extends:` is strong preset reuse. Diff-form `type-variants` are patch
operations for concision analysis, while selector breadth remains real
complexity.

## Acceptance Criteria

- [x] SQI scores root `extends:` wrappers from authored wrapper structure.
- [x] SQI reports diff-form `type-variants` without duplicate penalties.
- [x] Migration lineage exposes an explicit output plan.
- [x] Existing config wrappers strip template-bearing fields during migration.
- [x] Existing structural wrappers preserve structural deltas during migration.
- [ ] Multi-artifact embedded-root writes are available behind an explicit
      workflow command.
- [ ] Embedded-root writes mechanically prove resolved wrapper equivalence before
      writing.

## Changelog

- 2026-05-07: Initial active specification for SQI-audited root/wrapper migration.
