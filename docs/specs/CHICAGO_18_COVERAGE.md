# Chicago 18th / APA 8th Coverage Enhancement Specification

**Status:** Draft
**Date:** 2026-03-30
**Related:** [Chicago 18 CSL PR #7424](https://github.com/citation-style-language/styles/pull/7424), [APA 8th CSL PR #7510](https://github.com/citation-style-language/styles/pull/7510)

## Purpose

Document the scope of schema and engine enhancements needed for high-fidelity
support of the Chicago Manual of Style 18th edition and APA 8th edition citation
styles. Both styles were developed against the
[Zotero Test Items Library](https://www.zotero.org/groups/2205533/test_items_library)
(403 Chicago items, 357 APA 7th items), exercising ~50+ CSL variables that our
current test fixtures and schema do not fully cover.

This spec captures the gap analysis results and proposes a prioritized set of
schema extensions.

## Scope

**In scope:**
- New fields on existing reference types (Monograph, SerialComponent, etc.)
- New top-level `Event` reference type
- New `WorkRelation` relational model for `reviewed` and `original`
- New contributor roles
- Extension of `status` to more types (with i18n support)
- New genre values for existing types
- CSL→Citum conversion updates in `citum_migrate`

**Out of scope:**
- Actual style migration (Chicago 18th / APA 8th YAML styles)
- Engine rendering changes for new fields (follow-up work)

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

#### Batch 1: Multivolume / Serial Enrichment

| CSL Variable | Uses | Proposed Citum Field | Placement |
|----------|------|------|-----------|
| `volume-title` | 16 | `volume_title: Option<Title>` | `Monograph`, `CollectionComponent` |
| `part-number` | 6 | `part_number: Option<String>` | `Monograph`, `SerialComponent` |
| `part-title` | 2 | `part_title: Option<Title>` | `Monograph`, `SerialComponent` |
| `supplement-number` | 2 | `supplement_number: Option<String>` | `SerialComponent` |
| `chapter-number` | 7 | `chapter_number: Option<String>` | `Monograph`, `CollectionComponent` |

#### Batch 2: Event Type

A new top-level `InputReference::Event` variant for conferences, performances,
broadcasts, and recordings.

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
    /// Broadcaster or network.
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
| `references` | 5 | `references: Option<String>` | `Monograph`, `SerialComponent`, `Event` (appended bib note) |

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

Reviews are modeled as a relationship between a `SerialComponent` (the review)
and the work being reviewed.

| CSL Variable | Uses | Proposed Citum Field | Placement |
|----------|------|------|-----------|
| `reviewed-title` | 3 | `reviewed.title` | `SerialComponent` (via relation) |
| `reviewed-genre` | 1 | `reviewed.genre` | `SerialComponent` (via relation) |
| `section` (non-legal) | 9 | `section: Option<String>` | Extend to `SerialComponent` (magazine column/department) |
| `scale` | 1 | `scale: Option<String>` | `Monograph` (maps) |

### Relation Model: WorkRelation

The `WorkRelation` enum is untagged, allowing the relation to be either an 
explicit ID string or an embedded work object.

```rust
/// A relation between a review/commentary or reprint and the subject work.
#[serde(untagged)]
pub enum WorkRelation {
    /// The subject work is referenced by its ID.
    Id(RefID),
    /// The subject work is embedded inline.
    Embedded(Box<InputReference>),
}
```

**Relation Invariants:**
- **Short Names:** Fields are named `reviewed`, `original`, and `series`.
- **YAML Transparency:** Because of `#[serde(untagged)]`, the "work" wrapper 
  disappears in YAML/JSON (e.g., `reviewed: { type: book, title: ... }`).
- **No Cycles:** A work cannot be its own `original` or `reviewed` subject.
- **Non-Locating:** Relations are semantic links, not locator paths.
- **Identity:** Resolution via `id` is an engine-level optimization; embedded 
  data is the source of truth for snapshots.

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

- **Status:** Map to code if matching (`"forthcoming"`, etc.), otherwise to `status_display`.
- **Duration:** Convert CSL strings (e.g., `"1:30:00"`) to ISO 8601 (`"PT1H30M"`).
- **Original:** If flat `original_date`/`title` exist but `original` struct is missing, construct a minimal `original` reference. If both exist, the struct wins.
- **Parent:** `container-author` maps to `reviewed.author` (if a review) or `parent.author`.

## Acceptance Criteria

- [ ] All 18 missing CSL variables have corresponding Citum schema fields
- [ ] `Event` top-level type is defined with `series` relation
- [ ] `WorkRelation` model is implemented for `reviewed` and `original`
- [ ] `status` uses the Code + Display model for i18n
- [ ] `duration` uses ISO 8601 duration notation
- [ ] CSL→Citum conversion handles all new fields in `citum_migrate`
- [ ] `coverage-analysis.py` on chicago-18th.json reports 0 **unclassified** variables

## Changelog

- 2026-03-30: Standardized `WorkRelation` to untagged enum; fixed table naming inconsistencies.
- 2026-03-30: Refined Event model, added WorkRelation invariants, added i18n status and ISO duration.
- 2026-03-30: Updated to reflect Event as a top-level type and relational review model.
- 2026-03-30: Initial version from Chicago 18 PR analysis.
