# Numbering Semantics Specification

**Status:** Draft
**Date:** 2026-04-01
**Related:** `.beans/csl26-aew9--refine-numbering-semantic-distinctions.md`

## Purpose
Define the canonical semantics of Citum's numbering model so `number`, `report`, and `part` stop sharing the same storage and accessor behavior. This specification narrows the meaning of generic numbering, preserves dedicated fields for type-specific identifiers, and removes misleading numbering vocabulary that no longer matches the data model.

## Scope
In scope: canonical `NumberingType` meanings, accessor boundaries, conversion mapping for legacy CSL and biblatex imports, and rendering behavior for `number` versus `report-number`.

Out of scope: arbitrary user-defined numbering kinds, new locator vocabularies, and any redesign of `collection_number`/container inheritance.

## Design
### Canonical numbering vocabulary

`NumberingType` is reduced to numbering concepts that are genuinely shared across multiple reference classes:

- `volume`
- `issue`
- `number`
- `report`
- `part`
- `supplement`
- `chapter`
- `section`
- `edition`

`book` is removed. It is too ambiguous next to `MonographType::Book` and is not used by the current engine, migration paths, or styles.

### Semantic boundaries

- `number` is the generic document-level identifier for numbered monographs, collections, collection components, serial components, and classic works.
- `report` is reserved for report identifiers imported from report-like source records.
- `part` is reserved for true part semantics and is not the canonical target of shorthand `number`.
- `docket-number`, `patent-number`, `standard-number`, and `session-number` remain dedicated per-type fields rather than being normalized into shared numbering entries.

### Accessors

- `InputReference::number()` returns only generic `number` values from shorthand or canonical `NumberingType::Number`.
- `InputReference::report_number()` returns report identifiers from canonical `NumberingType::Report`.
- Existing dedicated getters and fields remain the source of truth for legal, patent, standard, and hearing identifiers.

### Canonicalization

- Flat shorthand `number` canonicalizes to `numbering: [{ type: "number", ... }]`.
- Report imports canonicalize to `numbering: [{ type: "report", ... }]`.
- Canonical serialization emits only the `numbering` array; shorthand fields remain input ergonomics only.
- This is a clean schema break: old overloaded numbering labels are not accepted as compatibility aliases.

### Import mapping

- Legacy CSL monograph-like `number` maps to `NumberingType::Number`.
- Legacy CSL report identifiers map to `NumberingType::Report`.
- Biblatex report `number` maps to `NumberingType::Report`.
- Biblatex and legacy serial article `number` continues to map to `NumberingType::Issue`.
- Collection or series numbering continues to map to `NumberingType::Volume`.

### Rendering

- `NumberVariable::Number` and `SimpleVariable::Number` render only generic numbering.
- `NumberVariable::ReportNumber` and `SimpleVariable::ReportNumber` render only report numbering.
- Locator mapping remains unchanged: `number -> LocatorType::Number`, `part-number -> LocatorType::Part`, `supplement-number -> LocatorType::Supplement`, `issue -> LocatorType::Issue`.

## Implementation Notes
- The schema bump for this change is `major`.
- `docs/schemas/` must be regenerated in the implementation commit because the schema crates change.
- Existing docs and examples that describe shorthand `number` as canonical `part` must be updated in the same implementation commit.

## Acceptance Criteria
- [ ] Generic shorthand `number` canonicalizes to `NumberingType::Number`.
- [ ] Report imports canonicalize to `NumberingType::Report`.
- [ ] `InputReference::number()` no longer returns report, docket, patent, standard, or hearing identifiers.
- [ ] `InputReference::report_number()` exists and is used by report rendering paths.
- [ ] `NumberingType::Book` is removed from the public schema.
- [ ] Styles using `report-number` continue to render correctly after the change.

## Changelog
- 2026-04-01: Initial draft.
