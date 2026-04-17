# Original Publication Relation Support Specification

**Status:** Draft
**Date:** 2026-04-17
**Related:** csl26-1nsh

## Purpose
Support the rendering of "original" publication metadata (original date, title, publisher) across all citeable Citum work/reference types. Currently, the `original` relation is only available on a subset of types (`Monograph`, `CollectionComponent`, `SerialComponent`), which causes data loss during CSL-JSON migration for other types (e.g., `Patent`, `Event`) and prevents correct rendering of the `original-date` variable in styles like Chicago 18th.

## Scope
In scope:
- Adding `original` field to all remaining citeable `InputReference` variants that represent works or documents.
- Updating accessors to handle all variants.
- Updating legacy CSL-JSON conversion to populate the new field.

Out of scope:
- Adding `original` to pure container records (`Collection`, `Serial`). These represent aggregating publications rather than citeable works in the current model, and this feature does not require modeling an "original collection" or "original serial" relation.
- Resolving `WorkRelation::Id` at the engine level for non-bibliographic contexts.
- Adding other relations (like `reviewed`) to more types.

## Design

### 1. Schema Expansion
The `original` field of type `Option<WorkRelation>` will be added to all citeable `InputReference` variants that currently lack it. This ensures that any work-like bibliographic item can track its original version while leaving pure container records unchanged.

- **Files**: `crates/citum-schema-data/src/reference/types/specialized.rs`, `crates/citum-schema-data/src/reference/types/legal.rs`
- **Impacted Structs**:
    - `Patent`, `Classic`, `Dataset`, `Standard`, `Software`, `Event`
    - `WorkCore` (covers `AudioVisualWork`)
    - `LegalCase`, `Statute`, `Treaty`, `Hearing`, `Regulation`, `Brief`
- **Explicit non-goals**:
    - `Collection`
    - `Serial`

### 2. Robust Accessors
The accessor methods in `InputReference` will be updated to match all variants.

- **Files**: `crates/citum-schema-data/src/reference/mod.rs`
- **Methods**: `original_date()`, `original_title()`, `original_publisher_str()`, `original_publisher_place()`.
- **Logic Change**: In `original_date()`, the embedded reference's date will be fetched via `p.csl_issued_date()` instead of `p.issued()`. This ensures that if the original work only has a `created` date (common for unpublished works or archival items), it is correctly picked up as the "original date".

### 3. Legacy CSL-JSON Migration
The CSL-JSON to Citum conversion logic will be updated to read `original-title` from the structured `csl_legacy::csl_json::Reference.original_title` field and extract the remaining legacy `original-*` data (`original-author`, `original-date`, `original-publisher`, `original-publisher-place`) from the `extra` map, then populate the `original` struct.

- **Files**: `crates/citum-schema-data/src/reference/conversion.rs`
- **Entry points**: extend the specialized and legal conversion paths that currently return work/document variants without `original` support. In practice this means `from_patent_ref`, `from_dataset_ref`, `from_standard_ref`, `from_event_ref`, the legal converters (`from_legal_case_ref`, `from_statute_ref`, `from_regulation_ref`, `from_treaty_ref`, `from_document_ref` for `Brief`/`Hearing`), and the shared monograph-style path already used by `Software`.
- **Normalization rule**: v1 intentionally normalizes migrated `original-*` metadata into an embedded `Monograph` via `relation_monograph`, even when the citing item is a patent, event, legal item, or audio-visual work.
- **Preserved fields**: the normalized embedded original preserves only the fields expressible by legacy CSL `original-*` data in current inputs: `title`, `author`, `issued`/effective original date, `publisher`, and `publisher-place`.
- **Out of scope for v1**: preserving the source subtype of the original work, modeling type-specific original-only fields, or guaranteeing round-tripping back to the same non-monograph subtype.

## Benefits
- **Full CSL Compatibility**: Resolves failures in benchmarks where `original-date` was previously ignored for non-book types.
- **Architectural Consistency**: Adheres to the Citum philosophy that `original` is a semantic relation to another work, not just a set of flat strings.
- **Improved Data Fidelity**: Captures reprints and original versions for all types of materials, improving Chicago and APA style accuracy.

## Risks & Considerations
- **Memory Usage**: Adding `Option<WorkRelation>` to more structs increases the memory footprint of the `InputReference` enum. Unlike `Option<Box<T>>`, `Option<WorkRelation>` is larger because `WorkRelation` has an `Id(RefID)` variant where `RefID` wraps a `String`. However, given that these fields are essential for CSL fidelity and usually `None`, the impact is considered acceptable for the required feature parity.
- **ID Resolution**: If `original` uses `WorkRelation::Id`, resolution currently happens at the engine level. The `original_date` accessor on `InputReference` returns `None` for IDs. This is acceptable as most legacy data from Zotero/CSL-JSON is migrated as `Embedded`.

## Acceptance Criteria
- [ ] `original` field is present in every citeable work/document `InputReference` variant except the out-of-scope pure container records `Collection` and `Serial`.
- [ ] `InputReference::original_date()` returns the correct date for all reference types when an `original` work is embedded.
- [ ] Legacy conversion populates `original` for representative non-structural types including `Patent`, `Event`, and one legal/document path.
- [ ] The spec and implementation treat migrated `original-*` metadata as a deliberately normalized embedded `Monograph` relation in v1.
- [ ] Chicago Author-Date 18th benchmark shows improvement in `original-date` rendering cases.

## Changelog
- 2026-04-17: Initial version.
