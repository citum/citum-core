---
# csl26-q4k2
title: 'Engine bug: initialize-with interaction with name rendering'
status: completed
type: bug
priority: high
created_at: 2026-03-16T20:15:00Z
updated_at: 2026-03-25T13:12:06Z
---

## Summary
OSCOLA and other note styles with `initialize-with: ""` (no dots) are rendering abbreviated names (e.g., "TS Kuhn") when `form: long` requests full names (e.g., "Thomas S Kuhn").

## Reproduction
Style: `styles/oscola.yaml`, citation template
- Option: `initialize-with: ""` (no dots for initialization)
- Template: `form: long` for author contributor
- Expected: "Thomas S Kuhn"
- Actual: "TS Kuhn"

## Root Cause
**Pre-fix:** Engine implicitly activated `NameForm::Initials` when `initialize-with` was present (matching CSL 1.0 spec). Migrator emitted `initialize-with` without `name-form: initials`, causing styles like OSCOLA (which use `initialize-with: ""` to mean "no dots") to accidentally trigger initialization.

**Post-fix:** `name-form: initials` is now the sole activator of initial rendering; `initialize-with` is a pure separator option. Migrator co-emits both fields where appropriate, ensuring backward compatibility without silent behavior changes.

## Impact Scope
- Affects OSCOLA, OSCOLA-no-ibid, and potentially other note styles with similar configuration
- Likely cross-portfolio issue affecting ~10-15 styles

## Proposed Fix
Review `citum-engine` name formatting logic to distinguish between:
1. Empty initialize-with (don't initialize, use full names)
2. Dot-based initialize-with (initialize with dots, e.g., ". ")
3. Non-dot initialize-with (initialize with custom separator, e.g., "")

Symbol path: `citum-engine::processor::rendering::name_formatting` or similar

## Summary of Changes

Co-emit `name_form: Initials` in style-level extractor when `initialize_with` is set. Added regression test for style-global initialize-with migration path.
