# Non-Standard Numbering And Locator Kinds Specification

**Status:** Active
**Date:** 2026-04-01
**Related:** `.beans/archive/csl26-jt8t--support-non-standard-numbering-locators.md`, `docs/specs/NUMBERING_SEMANTICS.md`, `docs/specs/LOCATOR_RENDERING.md`

## Purpose
Allow Citum to represent and render domain-specific numbering and locator labels without forcing every scholarly edge case into the fixed built-in vocabularies. This keeps the public wire shape simple while preserving the current separation between reference numbering metadata and citation pinpoint locators.

## Scope
In scope: string-backed custom numbering and locator kinds, custom `number:` template variables, locale-aware custom locator parsing/rendering, and schema/bindings/CLI support for the widened public shape.

Out of scope: automatic migration synthesis of custom kinds from CSL, a shared Rust abstraction merging numbering and locators, and changes to the higher-level meaning of existing built-in numbering or locator kinds.

## Design
### Public wire shape

The following fields accept either a known built-in keyword or a custom user-authored kebab-case string:

- `numbering[].type`
- `CitationLocator` segment `label`
- locale `locators` map keys
- `LocatorConfig.kinds` map keys
- `LocatorConfig.patterns[].kinds`
- `LocatorConfig.patterns[].order`
- `TemplateNumber.number`

Built-in values continue to serialize as their existing kebab-case strings. Unknown values normalize to kebab-case and round-trip as custom strings.

### Internal model

- `NumberingType`, `LocatorType`, and `NumberVariable` remain distinct Rust enums.
- Each enum gains `Custom(String)` and serializes as a plain string.
- The implementation does not introduce a shared numbering/locator base type in this change.

### Numbering behavior

- `NumberVariable::Custom(String)` renders by looking up a matching numbering entry on the reference.
- Existing dedicated accessors such as `number()`, `report_number()`, `volume()`, and `issue()` keep their current semantics.
- A new generic numbering lookup accessor is added for arbitrary numbering kinds.

### Locator behavior

- Locale-defined custom locator keys participate in alias generation during parsing.
- Custom locator rendering uses this fallback order:
  1. locale term for the requested form
  2. locale long/short fallback for the same custom kind
  3. raw custom identifier string
- No implicit aliases are invented beyond the locale data and the existing English built-ins.

## Implementation Notes
- This change is a schema bump: `major`.
- Because schema crates change, regenerate `docs/schemas/` in the implementation commit.
- Update CLI/style linting so custom locator requirements are validated against locale-provided custom terms.

## Acceptance Criteria
- [ ] Custom numbering kinds round-trip through schema serialization.
- [ ] Custom locator kinds round-trip through schema serialization.
- [ ] `TemplateNumber.number` accepts custom kinds and renders them from arbitrary numbering entries.
- [ ] Locale-defined custom locator labels parse and render through the existing locator pipeline.
- [ ] Existing built-in numbering and locator keywords remain unchanged on the wire.
- [ ] CLI/style linting handles custom locator requirements correctly.

## Changelog
- 2026-04-01: Activated with string-backed custom numbering and locator kinds.
- 2026-04-01: Initial draft.
