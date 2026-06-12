# Template Rendering Semantics Specification

**Status:** Active
**Date:** 2026-06-12
**Supersedes:** (none)
**Related:** csl26-y4o7, [TEMPLATE_V2.md](./TEMPLATE_V2.md),
[TEMPLATE_V3.md](./TEMPLATE_V3.md)

## Purpose

Define the render-time side effects of Citum templates. Template schema specs
define the shape of components and groups; this spec defines when rendering a
component claims a variable for duplicate suppression.

This exists because migrated CSL bibliography templates may contain fallback
components before the live component that should render the same variable. A
hidden fallback must not starve later visible output such as journal title,
volume, or page data.

## Scope

In scope:
- variable-once consumption for citation and bibliography templates
- explicit `suppress: true` behavior
- transactional variable consumption for template groups
- duplicate suppression between visible components

Out of scope:
- changing template YAML shape
- changing `type-variants` diff semantics
- changing bibliography section grouping
- adding a public API or schema field

## Design

### First Visible Occurrence Wins

Citum implements variable-once rendering as first-visible-occurrence semantics.
A component claims its variable only when it contributes non-empty, unsuppressed
output to the final rendered citation or bibliography entry.

Consequences:
- A visible first occurrence suppresses later occurrences of the same variable.
- A component that resolves to no value does not claim the variable.
- A component whose rendered value is whitespace-only does not claim the
  variable.
- A component with `suppress: true` does not claim the variable.

`suppress: true` is a rendering directive, not a semantic variable claim. This
must hold for top-level components, group children, and nested group children.

### Group Consumption Is Transactional

A template group evaluates its children as a unit. Child variable claims are
committed to the surrounding template only if the group itself contributes
visible output.

Consequences:
- If every child is empty or suppressed, the group renders nothing and no child
  variable is claimed.
- If the group has `suppress: true`, neither the group nor its children claim
  variables.
- If the group renders visible output, the visible children that contributed to
  that output claim their variables in document order.
- Nested groups follow the same rule recursively.

### Example

This migrated shape must render the live journal details:

```yaml
bibliography:
  template:
    - group:
        - title: parent-serial
        - date: issued
          form: year
      suppress: true
    - group:
        - title: parent-serial
        - number: volume
        - variable: page
      delimiter: ", "
```

The first group is suppressed, so `parent-serial` and `issued` do not claim
their variables. The second group may still render the journal title, volume,
and pages. If the first group were visible, `parent-serial` would claim its
slot and the later `parent-serial` would be skipped.

## Implementation Notes

The engine should decide consumption after value resolution and before
publishing component output to the surrounding template. For groups, evaluate
children against a scratch tracker and merge the scratch tracker into the
parent tracker only when the group emits visible output.

This keeps variable-once behavior tied to observable output rather than to
internal evaluation order.

## Acceptance Criteria

- [ ] A suppressed top-level component before a live same-variable component
      does not suppress the live component.
- [ ] A suppressed group before a live group does not consume variables used by
      the live group.
- [ ] Depth-1 and depth-2 suppressed group children have identical
      non-consuming behavior.
- [ ] A visible first occurrence still suppresses later duplicate components.
- [ ] `csl26-y4o7` records the semantic decision and regression evidence.

## Changelog

- 2026-06-12: Initial version.
