# Contributor Short-Name

Status: Active
Bean: csl26-ycyp

## Overview

`short-name` is an optional field on organizational/literal contributor names that provides
an acronym or abbreviated form (e.g. `WHO` for `World Health Organization`). When set and
integral citation name-memory is active, the engine renders the full name with the short name
on the first mention, then the short name alone on subsequent mentions.

## Input Schema

```yaml
author:
  name: World Health Organization
  short-name: WHO
```

`short-name` is valid only on `SimpleName` (organizational, institutional, or literal names).
It has no meaning on `StructuredName` — personal name abbreviation is handled by name
formatting options (`initialize-with`, `ContributorForm::Short`).

## Rendering Semantics

Short-name rendering is gated on the integral citation name-memory system
(the `integral-name-memory` style option). When the block is present, name memory
is active and:

| Integral name state | `short-name` present | Output |
|---------------------|----------------------|--------|
| `First`             | yes                  | see `short-name-display` option |
| `First`             | no                   | full name (unchanged) |
| `Subsequent`        | yes                  | short name only |
| `Subsequent`        | no                   | existing `subsequent-form` logic |
| non-integral        | any                  | full name always |

In bibliography output, the full name is always used regardless of `short-name`.

## Style Option: `short-name-display`

Placed under `integral-name-memory` in the style options block:

```yaml
options:
  integral-name-memory:
    short-name-display: full-then-parenthetical  # default
```

| Value | Output for first integral mention |
|-------|-----------------------------------|
| `full-then-parenthetical` (default) | `World Health Organization (WHO)` |
| `short-then-bracketed` | `WHO [World Health Organization]` |

## Relationship to Abbreviation Map

The abbreviation-map is a document-level post-render substitution applied to rendered
strings. `short-name` is an input-level field on the reference data. They are independent:

- Use `short-name` when the abbreviation is a stable property of the organization and
  should trigger first-mention parenthetical rendering.
- Use `abbreviation-map` for ad-hoc post-render substitutions (e.g. journal title
  abbreviations).

## Non-Goals

- `short-name` on `StructuredName` (personal names): not in scope; use `initialize-with`.
- Per-language `short-name` variants: not in scope; `short-name` is a plain string.
- Fuzzy or partial matching: the engine uses the short name verbatim.
