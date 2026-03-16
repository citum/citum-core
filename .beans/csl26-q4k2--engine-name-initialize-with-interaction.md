---
# csl26-q4k2
title: "Engine bug: initialize-with interaction with name rendering"
status: todo
type: bug
priority: high
created_at: 2026-03-16T20:15:00Z
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
Engine appears to be treating `initialize-with: ""` as "render abbreviated initials without dots" rather than "don't add dots after full names". This is a semantic mismatch in how the initialize-with flag is interpreted.

## Impact Scope
- Affects OSCOLA, OSCOLA-no-ibid, and potentially other note styles with similar configuration
- Likely cross-portfolio issue affecting ~10-15 styles

## Proposed Fix
Review `citum-engine` name formatting logic to distinguish between:
1. Empty initialize-with (don't initialize, use full names)
2. Dot-based initialize-with (initialize with dots, e.g., ". ")
3. Non-dot initialize-with (initialize with custom separator, e.g., "")

Symbol path: `citum-engine::processor::rendering::name_formatting` or similar
