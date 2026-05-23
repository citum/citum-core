# Integral Citation Name Memory

Status: Active
Bean: csl26-d7uz

## Overview

The `integral-name-memory` style option configures **full-then-short narrative-name memory** for integral (in-text, narrative) citations. The presence of the block enables the memory; absence disables it. There is no `enabled:` field and no `rule:` enum.

The feature targets the *author-name portion* of integral citations only. It does not affect parenthetical citations, notes (except when explicitly opted in via `contexts`), bibliographies, locators, years, or non-author contributor roles.

## When to use it

Only one mainstream English-language pattern needs this block:

- **MLA-style body text**: the author's full name on the first integral mention in body text; the surname on subsequent integral mentions.

Styles that always render the surname for narrative citations (APA 7, Chicago author-date) need *no* block — `contributor: author, form: short` in the integral template is sufficient.

## YAML shape

```yaml
options:
  integral-name-memory:
    scope: document            # document | chapter | section
    contexts: body-only        # body-only | body-and-notes
    subsequent-form: short     # short | family-only
    short-name-display: full-then-parenthetical
```

All fields are optional. Defaults: `scope: document`, `contexts: body-only`, `subsequent-form: short`, `short-name-display: full-then-parenthetical`.

### Field semantics

| Field | Meaning |
|---|---|
| `scope` | Where memory resets. `document` keeps one scope for the whole document; `chapter` resets on chapter boundaries; `section` resets on section boundaries. |
| `contexts` | Whether note citations participate in memory. `body-only` is the safe default; `body-and-notes` shares one memory across both. |
| `subsequent-form` | The contributor form rendered for subsequent integral mentions. `short` keeps non-dropping particles ("van Beethoven"); `family-only` strips them ("Beethoven"). |
| `short-name-display` | How an organisational `short-name` is displayed on the first mention. See `docs/specs/SHORT_NAME.md`. |

## Embedded styles

| Style | Configuration | Notes |
|---|---|---|
| MLA 9 | `contexts: body-only`, `subsequent-form: short` | Full name first mention, surname (with particle) subsequent in body text. Notes follow MLA's distinct note convention and are intentionally out of scope. |
| APA 7 | no block | Surname only via integral template; no memory engaged. |
| Chicago author-date | no block | Same as APA. |

## Migration from earlier syntax

The pre-2026-05 schema used a different key (`integral-names`) and an extra `rule` enum. Migrating an external style is mechanical:

| Old | New |
|---|---|
| `integral-names:` (top-level YAML key) | `integral-name-memory:` |
| `rule: full-then-short` (sub-key) | remove — block presence is the on-switch |
| `rule: short-only` (sub-key) | remove the entire block — there is no memory to opt out of |

Other sub-keys (`scope`, `contexts`, `subsequent-form`, `short-name-display`) keep their names and values.

## Document-level overrides

A document's frontmatter may override the style's policy through `integral-name-memory:` with the same fields plus an `enabled: false` switch to suppress memory entirely for that document. See `DocumentIntegralNameOverride` in `crates/citum-engine/src/processor/document/types.rs`.

## Engine implementation pointers

- `crates/citum-engine/src/processor/document/integral_names.rs` — `annotate_integral_name_states` early-returns when the resolved memory config is `None`; that is the off-switch.
- `crates/citum-engine/src/values/contributor/mod.rs` — `apply_integral_subsequent_form` engages only when memory is configured and the citation item carries `IntegralNameState::Subsequent`.
- `crates/citum-engine/src/values/contributor/names.rs` — short-name rendering driven by the same memory config.

## Non-goals

- Notes-style "first full / subsequent ibid or short title": handled by note-citation mechanics elsewhere (Chicago notes, OSCOLA position overrides).
- Per-role memory (editors, translators): not in scope; memory applies to `author` only.
- Year/locator memory: out of scope.
