# Config-Only Profile Overrides Specification

**Status:** Active
**Date:** 2026-04-21
**Related:** `TEMPLATE_V2.md`, `STYLE_PRESET_ARCHITECTURE.md`, `STYLE_TAXONOMY.md`, bean `csl26-xt7k`, bean `csl26-nrkn`, bean `csl26-rwgi`

## Purpose

This specification defines an alternative to Citum's current `extends:`
contract for style profiles. Under this model, a profile still selects a
guide-backed base style, but it may not merge or replace template-bearing
structures. Instead, all supported variation must be expressed through
explicit, typed configuration options. Any guide-backed child that still needs
structural template changes becomes a new base or an independent style rather
than a merged wrapper.

This draft extends the same simplification direction already taken in
`TEMPLATE_V2.md` and PR 426 ("template simplification"), where per-component
template override machinery was removed in favor of spec-level `type-variants`
and style-level configuration. The question here is whether the same principle
should apply one level higher, at style inheritance boundaries.

## Scope

In scope:

- the resolution contract for styles that declare `extends:`
- schema and validation changes needed to make profile overrides config-only
- the typed option surface required to absorb recurring family-level variation
- taxonomy and registry implications of removing template merging from profiles

Out of scope:

- arbitrary structural parameterization of templates
- generic stringly-typed profile maps or plugin-style option bags
- preserving current bulky profiles as profiles if they still require template
  edits
- implementation of this design in the current PR

## Design

### 0. Terms

**Template-bearing field** means any field whose schema type is `Template`, or
any container/variant type that can contain one or more `Template` values.
Profiles must not override template-bearing fields.

This rule applies both to the currently known paths listed below and to any
future schema field that is typed as `Template` or can contain `Template`
values indirectly. New template-bearing fields are non-overridable by profiles
unless a future spec explicitly changes that rule.

### 1. Profile Contract

Under this model, `extends:` remains the way a style selects a guide-backed
base, but the meaning of a profile changes:

- a `base` owns complete citation and bibliography templates
- a `profile` inherits those templates intact
- a `profile` may override local identity/metadata plus `options.profile`
- a `profile` may not override any template-bearing field

In practical terms, a profile must not supply local values for:

- top-level `templates`
- `citation.template`
- `citation.integral.template`
- `citation.non-integral.template`
- `citation.type-variants`
- `citation.integral.type-variants`
- `citation.non-integral.type-variants`
- `bibliography.template`
- `bibliography.type-variants`
- any future template-bearing field

If a publisher or standards child style requires one of those edits, the style
stops being a profile under this model. The correct fix is to:

1. author a new family base, if the child represents a reusable structural
   branch, or
2. keep the style independent, if the structural change is one-off

### 1.1 Local Identity vs. Inherited Structure

A profile remains a distinct public style handle. It keeps its own local
`info.id`, title, summary, and other user-facing metadata even though it
inherits rendering structure from the base.

The inheritance rule is therefore:

- identity is local to the profile
- rendering structure is inherited from the base
- profile-local metadata does not imply permission to override template-bearing
  fields

### 2. Resolution Semantics

`Style::into_resolved()` becomes a metadata-and-config overlay operation, not a
general structural merge.

Resolution order:

1. Load the selected base style.
2. Overlay profile-local identity/metadata fields.
3. Overlay `options.profile` values.
4. Validate that the profile did not supply forbidden template-bearing fields.
5. Validate that the selected base supports every requested profile option.
6. Return the effective style with the base templates unchanged.

This removes the current deep-merge/replace-wholesale behavior for profile
template subtrees entirely. Explicit `null` remains valid only for allowed
config fields; it is not a legal way to clear inherited templates or inherited
type variants in a profile.

For `options.profile`, overlay semantics are per-leaf rather than replace-whole
for nested structs. Attempting to clear a template-bearing field with `null`, or
using `null` in a way that violates the profile-axis schema, is a validation
error rather than a silently ignored request.

### 3. Typed Profile Options

To replace template edits, Citum adds a dedicated typed configuration surface
under `options.profile`.

```yaml
extends: springer-basic-core
options:
  profile:
    citation-label-wrap: brackets
    bibliography-label-mode: numeric
    date-position: after-author
    volume-pages-delimiter: colon
```

`options.profile` is a strongly typed struct, not a free-form map. New fields
may be added only when all of the following are true:

- the variation has stable rendering semantics
- the variation appears across multiple styles or a large family, not one file
- the variation is explainable without referencing hidden template structure
- the axis is orthogonal to existing profile axes where practical

The initial option families for this model are:

- citation label presentation
  - `citation-label-wrap`: `none | parentheses | brackets | superscript`
  - `citation-group-delimiter`: `comma | semicolon | space`
- bibliography label presentation
  - `bibliography-label-mode`: `none | numeric | author-date`
  - `bibliography-label-wrap`: `none | parentheses | brackets`
- entry sequencing and punctuation
  - `date-position`: `after-author | after-title | terminal`
  - `volume-pages-delimiter`: `comma | colon | space`
  - `title-terminator`: `period | comma | none`
- contributor list behavior
  - `name-list-profile`: references existing typed contributor presets
  - `repeated-author-rendering`: `full | dash | dash-with-space`

This list is intentionally bounded. If a proposed option needs to describe a
whole template fragment rather than a stable behavior axis, it does not belong
in `options.profile`.

The default policy is conservative: the first implementation should cover only
the small set of recurring axes already evidenced by current Springer,
Elsevier, and Taylor & Francis family analysis. Broader axis mining is follow-
up work, not part of the initial contract.

### 4. Base Capabilities

Every embedded base exposes the set of `options.profile` axes it supports.
Loading a profile with an unsupported axis is a validation error.

This keeps profile authoring explicit:

- a style cannot silently request a knob that its base ignores
- two family bases can support different subsets of the shared profile options
- base authors must declare which behavior axes are part of the base contract

The capability list is runtime metadata in `StyleBase`, not user-authored YAML.
Schema validation ensures shape; base capability validation ensures semantic
fit.

Conceptually, a base capability declaration looks like:

```yaml
profile-capabilities:
  citation-label-wrap:
    values: [none, parentheses, brackets]
  bibliography-label-mode:
    values: [none, numeric]
```

This is illustrative only. The actual source of truth is compiled metadata, not
raw YAML.

Validation distinguishes three error cases:

- the engine does not recognize the requested profile axis
- the selected base does not support that axis
- the base supports the axis, but not the requested value

### 5. Family Roots and Public Handles

This model prefers hidden family roots over public-to-public chaining.

Example:

- `springer-basic-core` is a hidden embedded base
- `springer-basic-author-date` extends `springer-basic-core` with profile
  options only
- `springer-basic-brackets` extends `springer-basic-core` with profile options
  only

The same pattern applies to Elsevier and Taylor & Francis families where the
house style is real but the public handles represent sibling guide profiles.

This is a deliberate consequence of removing template merging from profiles:
the system needs more reusable structural roots, but it no longer needs child
styles to restate large template blocks just to tweak punctuation or label
mode.

The normative preference is:

- public profiles should extend hidden family roots
- public-to-public `extends:` chains should be avoided
- a public-to-public chain needs an explicit documented exception

Migration consequence: some currently bulky "semantic profiles" will need to be
split into a hidden `*-core` base plus public config-only siblings if the
project adopts this model.

### 6. Taxonomy and Registry Consequences

If this model is adopted, the taxonomy definition of `profile` becomes stricter
than the current one:

- `profile` means guide-backed parentage plus config-only delta
- a style with real parentage but structural template edits is not a profile in
  implementation or taxonomy
- those structurally distinct children become either `base` or `independent`

Normative taxonomy summary:

- `profile`: declares `extends:`, has guide-backed parentage, and defines no
  template-bearing local fields
- `base`: owns templates and is intended for reuse by other styles
- `independent`: owns templates and does not declare `extends:`

The practical result is:

- fewer bulky "semantic profiles"
- more family bases
- a clearer split between parametric variation and structural variation

### 7. Authoring Examples

Minimal sibling profiles:

```yaml
info:
  id: springer-basic-author-date
extends: springer-basic-core
options:
  profile:
    bibliography-label-mode: none
    date-position: after-author
```

```yaml
info:
  id: springer-basic-brackets
extends: springer-basic-core
options:
  profile:
    citation-label-wrap: brackets
    bibliography-label-mode: numeric
    volume-pages-delimiter: colon
```

Invalid profile under this model:

```yaml
extends: springer-basic-core
bibliography:
  type-variants:
    article-journal:
      - text: "not allowed here"
```

That file must either become a new base or remain independent.

Invalid profile using `null` as a template-clearing mechanism:

```yaml
extends: springer-basic-core
bibliography:
  template: ~
```

This is also invalid. Profiles may not clear inherited template-bearing fields.

## Implementation Notes

This design is intentionally simpler than the current profile merge model, but
the simplification moves work rather than eliminating it.

- Resolver complexity drops because profile resolution no longer merges
  template subtrees or relies on replace-whole array behavior.
- Schema and engine complexity rise because recurring wrapper differences must
  become first-class typed options.
- Family-root authoring becomes more important because structurally distinct
  siblings can no longer piggyback on public bases through template deltas.
- Registry validation becomes more important because claimed taxonomy must match
  actual structure.

Historically, this aligns more closely with the project's January 31, 2026
style-reuse direction, which favored presets/configuration and self-describing
styles over deep alias chains. It also follows the same simplification logic as
Template V2 and PR 426, which removed per-component template overrides instead
of preserving a more flexible but harder-to-reason-about merge model. The key
difference is that this draft applies that rule at the style-profile boundary
rather than inside a single template tree.

## Acceptance Criteria

- [ ] Profiles that declare `extends:` and also provide template-bearing fields
      fail validation with a clear error.
- [ ] Profiles that use `null` to clear template-bearing fields fail validation
      with a clear error.
- [ ] `Style::into_resolved()` for profiles becomes a metadata/config overlay
      operation; no profile path performs structural template merging.
- [ ] `options.profile` is a typed schema surface with documented initial axes
      and no generic string map escape hatch.
- [ ] Each embedded base declares which profile axes it supports, and
      unsupported axes are rejected at load time.
- [ ] Validation distinguishes unknown axis, unsupported axis, and unsupported
      value errors.
- [ ] Taxonomy and registry rules define `profile` as config-only and require
      structural children to become new bases or independent styles.
- [ ] Linting/validation rejects any style classified as a `profile` when its
      structure includes template-bearing local fields.

## Changelog

- 2026-04-21: Initial draft.
- 2026-04-21: Activated with the first resolver/schema implementation.
