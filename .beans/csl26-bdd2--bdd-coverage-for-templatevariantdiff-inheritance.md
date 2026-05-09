---
# csl26-bdd2
title: Add comprehensive BDD coverage for TemplateVariantDiff inheritance chains
status: todo
type: task
priority: high
created_at: 2026-05-09T00:00:00Z
updated_at: 2026-05-09T00:00:00Z
---

The declarative template V3 diff system (`TemplateVariantDiff`) is a core 
architectural strength, but it introduces significant cognitive and logical 
complexity when styles utilize deep inheritance or multiple structural overlays.

## Scope

- Design a suite of Behavioral Driven Development (BDD) tests specifically targeting 
  complex inheritance scenarios in `merge_style_overlay`.
- Test scenarios should include:
    - Multiple levels of style inheritance with conflicting variant diffs.
    - Partial overrides of complex template components.
    - Interaction between global options and type-specific variant diffs.
- Ensure that the resolution of structural exceptions remains deterministic and 
  predictable as the style library grows.

## Rationale

As more complex styles are migrated to the Citum schema, the inheritance logic 
becomes a critical failure point. Robust BDD coverage ensures that structural 
integrity is maintained during refactoring or schema evolution.
