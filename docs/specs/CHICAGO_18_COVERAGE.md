# Chicago 18th / APA 8th Coverage Enhancement Specification

**Status:** Active
**Date:** 2026-03-31
**Related:** [Chicago 18 CSL PR #7424](https://github.com/citation-style-language/styles/pull/7424), [APA 8th CSL PR #7510](https://github.com/citation-style-language/styles/pull/7510), [Generalized WorkRelation Spec](./GENERALIZED_RELATIONAL_CONTAINER_MODEL.md)

## Purpose

Document the scope of schema and engine enhancements needed for high-fidelity support of the Chicago Manual of Style 18th edition and APA 8th edition citation styles. Both styles were developed against the [Zotero Test Items Library](https://www.zotero.org/groups/2205533/test_items_library) (403 Chicago items, 357 APA 7th items), exercising ~50+ CSL variables that our current test fixtures and schema do not fully cover.

This spec captures the gap analysis results and proposes a prioritized set of schema extensions, building upon the new recursive `container` model introduced in the Generalized WorkRelation specification.

## Scope

**In scope:**
- Adoption of recursive `container` relations to replace flat hierarchy fields.
- New top-level `Event` reference type.
- Implementation of `WorkRelation` relational model for `reviewed`, `original`, and `series`.
- New contributor roles.
- Extension of `status` to more types (with i18n support).
- New genre values for existing types.
- CSL→Citum conversion updates in `citum_migrate`.

**Out of scope:**
- Actual style migration (Chicago 18th / APA 8th YAML styles).
- Engine rendering changes for new fields (follow-up work).

## Design

### Coverage Analysis Summary

Running `scripts/coverage-analysis.py` on the Chicago 18th test corpus (403 items):

| Category | Count | Description |
|----------|-------|-------------|
| ✅ Covered | 32 | CSL variables with direct Citum schema mapping |
| ❌ Missing | 18 | CSL variables with no Citum equivalent |
| ⚠️ Partial | 21 | Variables mapped but with caveats or incomplete type coverage |
| ❓ Unmapped | 11 | Variables not yet classified in the analysis script |

### Missing Variables — Proposed Schema Additions

#### Batch 1: Multivolume / Serial Enrichment via Recursion

*Revision Note: This batch originally proposed adding flat fields like `volume_title` and `part_number`. To align with FRBR principles and future-proof the schema, Citum will instead adopt a recursive container model.*

CSL JSON fixtures show that variables like generic `number`, `part-number`, and `part-title` typically define the identity of the physical document being cited. In Citum, these are mapped directly to the document's own `title` plus either generic `numbering` or true part numbering, while the larger work is defined as a `container`.

| CSL Variable | CSL Uses | Citum Relational Strategy |
| :--- | :--- | :--- |
| `part-number` | 6 | **Document Attribute:** Mapped to the cited item's document-level numbering, using generic `number` or true `part` semantics as appropriate. |
| `part-title` | 2 | **Document Attribute:** Mapped to the `title` of the cited item. |
| `volume-title` | 16 | **Container Attribute:** Mapped to the `title` of the item's `container` (WorkRelation). |
| `chapter-number` | 7 | **Document Attribute:** Mapped to the `number` of the Chapter item. |

#### Batch 2: Event Type

A new top-level `InputReference::Event` variant for conferences, performances, broadcasts, and recordings.

**Rationale for Event over Monograph:**
Events are semantically distinct from monographs (books/reports) in that they are primarily *episodes in time* rather than *physical or digital works*. While a monograph is "published", an event is "held" or "performed". Chicago 18th and APA 8th increasingly treat these as first-class entities with specific roles (organizer, performer) and locators (venue, network).

```rust
/// Event metadata for conferences, performances, broadcasts, and recordings.
pub struct Event {
    /// Unique identifier.
    pub id: Option<RefID>,
    /// Event name (e.g., conference title, performance name).
    pub title: Option<String>,
    /// Recurring event series.
    pub series: Option<WorkRelation>,
    /// Event location (city, venue).
    pub location: Option<String>,
    /// Event date.
    pub date: Option<EdtfString>,
    /// Event genre (e.g., "conference", "performance", "broadcast", "talk").
    pub genre: Option<String>,
    /// Broadcaster, network, or streaming platform.
    pub network: Option<String>,
    /// Performer(s) or presenter(s).
    pub performer: Option<Contributor>,
    /// Organizer or sponsor.
    pub organizer: Option<Contributor>,
}
```

**Genre Vocabulary (Event):**
`"conference"`, `"talk"`, `"panel"`, `"performance"`, `"broadcast"`, `"reading"`.

| CSL Variable | Uses | Proposed Citum Field |
|----------|------|------|
| `event-title` | 10 | `Event.title` |
| `event-place` / `event-location` | 9 | `Event.location` |
| `event-date` | 13 | `Event.date` |

#### Batch 3: Status / Meta Fields

| CSL Variable | Uses | Proposed Citum Field | Notes |
|----------|------|------|-------|
| `status` | 22 | `status: Option<String>` | Controlled code: `"forthcoming"`, `"in-press"`, `"submitted"`, `"in-preparation"` |
| `status` (prose) | - | `status_display: Option<String>` | Verbatim override (bypasses localization) |
| `available-date` | 11 | `available_date: Option<EdtfString>` | `Monograph`, `SerialComponent`, `Event` |
| `dimensions` (size) | 20 | `size: Option<String>` | Physical dimensions (maps, art) |
| `dimensions` (time) | 7 | `duration: Option<String>` | ISO 8601 duration (e.g., `PT1H30M`) |
| `references` | 5 | `references: Option<String>` | Appended bibliography note (not for annotations) |

#### Batch 4: Contributor Roles

| CSL Variable | Uses | Proposed Citum Field | Placement |
|----------|------|------|-----------|
| `narrator` | 3 | `narrator: Option<Contributor>` | `Monograph`, `Event` |
| `compiler` | 2 | `compiler: Option<Contributor>` | `Monograph`, `Collection` |
| `producer` | 3 | `producer: Option<Contributor>` | `Monograph`, `Event` |
| `host` | 0* | `host: Option<Contributor>` | `SerialComponent`, `Event` (podcast host) |
| `container-author` | 4 | `reviewed.author` | `SerialComponent` (via relation) |
| `reviewed-author` | 8 | `reviewed.author` | `SerialComponent` (via relation) |
| `composer` | 4 | `composer: Option<Contributor>` | `Monograph`, `Event` |
| `performer` | 1 | `performer: Option<Contributor>` | `Monograph`, `Event` |

\* `host` is not used in the current corpus but is a standard CSL role.

#### Batch 5: Review / Relation Fields

Reviews are modeled as a relationship between a `SerialComponent` (the review) and the work being reviewed.

| CSL Variable | Uses | Proposed Citum Field | Placement |
|----------|------|------|-----------|
| `reviewed-title` | 3 | `reviewed.title` | `SerialComponent` (via relation) |
| `reviewed-genre` | 1 | `reviewed.genre` | `SerialComponent` (via relation) |
| `section` (non-legal) | 9 | `section: Option<String>` | Extend to `SerialComponent` (magazine column/department) |
| `scale` | 1 | `scale: Option<String>` | `Monograph` (maps) |

### Relation Model: WorkRelation

The `WorkRelation` enum is untagged, allowing the relation to be either an explicit ID string or an embedded work object.

```rust
/// A relation to another bibliographic entity.
/// Untagged in serde to allow either an inline object or a string ID reference.
#[serde(untagged)]
pub enum WorkRelation {
    /// The target work is referenced by its ID.
    Id(RefID),
    /// The target work is embedded inline.
    Embedded(Box<InputReference>),
}
```

**YAML Example (Reviewed Work - Embedded):**
```yaml
id: smith2026
type: article
title: A Review of The Great Gatsby
author: Jane Smith
reviewed:
  type: book
  title: The Great Gatsby
  author: F. Scott Fitzgerald
```

**YAML Example (Reviewed Work - via Citekey/ID):**
```yaml
id: smith2026
type: article
title: A Review of The Great Gatsby
author: Jane Smith
reviewed: fitzgerald1925
```

**Relation Invariants:**
- **Short Names:** Fields are named `reviewed`, `original`, and `series`.
- **YAML Transparency:** Because of `#[serde(untagged)]`, the "work" wrapper disappears in YAML/JSON.
- **No Cycles:** A work cannot be its own `original` or `reviewed` subject.
- **Non-Locating:** Relations are semantic links, not locator paths.
- **Identity:** Resolution via `id` is an engine-level optimization; embedded data is the source of truth for snapshots.

#### Batch 6: Original Publication

The `original` field (WorkRelation) replaces flat `original_*` fields.

| CSL Variable | Uses | Proposed Citum Field | Notes |
|----------|------|------|-------|
| `original-publisher` | 5 | `original.publisher` | Via WorkRelation |
| `original-date` | - | `original.issued` | Deprecates flat `original_date` |
| `original-title` | - | `original.title` | Deprecates flat `original_title` |

### Genre / Subtype Decisions

These CSL types map to Citum genres or the new `Event` type:

| CSL Type Override | Citum Mapping |
|----------|------|
| `performance` | `Event` with `genre: "performance"` |
| `speech` / `presentation` | `Event` with `genre: "talk"` or `"conference"` |
| `broadcast` | `Event` with `genre: "broadcast"` (live/stream) |
| `broadcast` (recorded) | `SerialComponent` (episode) |
| `pamphlet` | `Monograph` with `genre: "pamphlet"` |
| `musical_score` | `Monograph` with `genre: "musical-score"` |
| `periodical` | `Serial` (title-level citation) |
| `classic` | Already has dedicated `Classic` type ✅ |

## Upsampler Precedence (CSL→Citum)

- **Hierarchical Variables:** Flat fields (`volume`, `volume-title`, `part`, `part-title`) are parsed and up-sampled into deeply nested `container` tree structures using `WorkRelation`, with the part identifier remaining on the document itself.
- **Status:** Map to code if matching (`"forthcoming"`, etc.), otherwise to `status_display`.
- **Duration:** Convert CSL strings (e.g., `"1:30:00"`) to ISO 8601 (`"PT1H30M"`).
- **Original:** If flat `original_date`/`title` exist but `original` struct is missing, construct a minimal `original` reference. If both exist, the struct wins.
- **Parent:** `container-author` maps to `reviewed.author` (if a review) or `container.author`.

## Acceptance Criteria

- [ ] `Event` top-level type is defined with `series` relation
- [ ] `WorkRelation` model is implemented for `reviewed` and `original`
- [ ] `status` uses the Code + Display model for i18n
- [ ] `duration` uses ISO 8601 duration notation
- [ ] CSL→Citum conversion handles all new fields and recursive `container` relationships in `citum_migrate`
- [x] `coverage-analysis.py` reports 0 missing variables on `chicago-18th.json`, `apa-7th.json`, and `apa-test.json`

## Changelog

- 2026-04-09: Promoted to Active and aligned the coverage-analysis exit criteria with the bean completion gate.
- 2026-03-31: Mandated recursive `container` model per Generalized WorkRelation spec; "Part = Document" modeling.
- 2026-03-31: Refined CSL variable parsing; added YAML examples; rationalized Event type.
- 2026-03-30: Standardized `WorkRelation` to untagged enum; fixed table naming inconsistencies.
- 2026-03-30: Refined Event model, added WorkRelation invariants, added i18n status and ISO duration.
- 2026-03-30: Updated to reflect Event as a top-level type and relational review model.
- 2026-03-30: Initial version from Chicago 18 PR analysis.
