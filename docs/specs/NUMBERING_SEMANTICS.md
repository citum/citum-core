# Numbering Semantics Specification

**Status:** Active
**Date:** 2026-04-01
**Related:** `.beans/archive/csl26-aew9--refine-numbering-semantic-distinctions.md`, `.beans/csl26-7edf--add-dedicated-partsupplementprinting-number-fields.md`

## Purpose
Define the canonical semantics of Citum's numbering model so `number`, `report`, and `part` stop sharing the same storage and accessor behavior. This specification narrows the meaning of generic numbering, preserves dedicated fields for type-specific identifiers, and removes misleading numbering vocabulary that no longer matches the data model.

## Scope
In scope: canonical `NumberingType` meanings, accessor boundaries, conversion mapping for legacy CSL and biblatex imports, rendering behavior for `number` versus `report-number`, and shorthand input fields for `part-number`, `supplement-number`, and `printing-number`.

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
- `printing`
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

## Part, Supplement, and Printing Numbers

### Purpose

`part-number`, `supplement-number`, and `printing-number` are CSL number variables
that have no dedicated storage in the data model and previously resolved to `None` in
the engine. This section specifies how they are stored, ingested, and resolved.

### Vocabulary

- `part` — a numbered subdivision of a work that is itself a distinct publication unit
  (e.g., Part 2 of a multi-part volume). Not to be confused with the generic `number`.
- `supplement` — an identified supplement issue or addendum to a serial work
  (e.g., Supplement 3 of a journal volume). Non-standard in CSL 1.0; present in CSL-M.
- `printing` — the printing run of a monograph, distinct from edition
  (e.g., "3rd printing" of a first edition). Standard in CSL 1.0.

### Storage

All three are stored as `Numbering` entries in the canonical `numbering` vec using
`NumberingType::Part`, `NumberingType::Supplement`, and `NumberingType::Printing`
respectively. No dedicated canonical storage fields are added; canonical
serialization continues to use the existing `numbering` model. For authoring and
deserialization ergonomics, some reference structs accept input-only shorthand
fields (`part_number`, `supplement_number`, `printing_number`) that are normalized
into `numbering` entries and not preserved as separate canonical data.

### Shorthand input fields

For authoring ergonomics, the following flat shorthand fields are accepted on all
structural reference types (`Monograph`, `Collection`, `CollectionComponent`,
`SerialComponent`, `Classic`) and are normalized into `numbering` entries on
deserialization, following the same pattern as `volume`, `issue`, `edition`,
and `number`:

| Shorthand field     | Normalizes to                                    |
|---------------------|--------------------------------------------------|
| `part-number`       | `numbering: [{type: part, value: …}]`            |
| `supplement-number` | `numbering: [{type: supplement, value: …}]`      |
| `printing-number`   | `numbering: [{type: printing, value: …}]`        |

Shorthand fields are input-only. Canonical serialization emits only the `numbering`
array.

### Engine resolution

The engine resolves these `NumberVariable` variants via `reference.numbering_value()`:

- `NumberVariable::PartNumber` → `NumberingType::Part`
- `NumberVariable::SupplementNumber` → `NumberingType::Supplement`
- `NumberVariable::PrintingNumber` → `NumberingType::Printing`

### Ingestion

| Source | Field | Maps to |
|--------|-------|---------|
| CSL JSON `extra` | `part-number` | `NumberingType::Part` |
| CSL JSON `extra` | `supplement-number` | `NumberingType::Supplement` |
| CSL JSON variable | `printing-number` | `NumberingType::Printing` |
| biblatex | `part` | `NumberingType::Part` |
| biblatex | no dedicated field | `supplement-number` and `printing-number` have no biblatex source; export via `number`/`note` as appropriate |

### YAML examples

```yaml
# Monograph: a specific printing of a first edition
type: book
title: The Elements of Style
edition: "1st"
printing-number: "3"
```

```yaml
# Serial component: a supplement issue
type: article-journal
title: Special Report on Climate Indicators
container-title: Nature Climate Change
volume: "14"
supplement-number: S1
```

```yaml
# Collection component: a numbered part within a multi-part volume
type: chapter
title: Origins of the Universe
container-title: Encyclopedia of Cosmology
part-number: "2"
pages: 45–67
```

```yaml
# Canonical form (as serialized; shorthands normalized away)
numbering:
  - type: part
    value: "2"
  - type: printing
    value: "3"
```

## Implementation Notes
- The schema bump for this change is `patch` (adds optional fields/variants only).
- `docs/schemas/` must be regenerated in the implementation commit because the schema crates change.
- Existing docs and examples that describe shorthand `number` as canonical `part` must be updated in the same implementation commit.

## Acceptance Criteria
- [ ] Generic shorthand `number` canonicalizes to `NumberingType::Number`.
- [ ] Report imports canonicalize to `NumberingType::Report`.
- [ ] `InputReference::number()` no longer returns report, docket, patent, standard, or hearing identifiers.
- [ ] `InputReference::report_number()` exists and is used by report rendering paths.
- [ ] `NumberingType::Book` is removed from the public schema.
- [ ] Styles using `report-number` continue to render correctly after the change.
- [ ] `part-number`, `supplement-number`, `printing-number` shorthand fields normalize to `numbering` entries on all structural types.
- [ ] `NumberVariable::PartNumber`, `SupplementNumber`, `PrintingNumber` resolve from `numbering` in the engine.
- [ ] `printing-number` CSL JSON variable is ingested to `NumberingType::Printing`.
- [ ] `NumberingType::Printing` is present in the public schema.

## Changelog
- 2026-04-28: Added part/supplement/printing-number section (storage, shorthands, engine resolution, ingestion, YAML examples).
- 2026-04-01: Activated with the clean-break numbering semantics implementation.
- 2026-04-01: Initial draft.
