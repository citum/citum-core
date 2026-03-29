# Archival and Unpublished Source Support

**Status:** Active
**Date:** 2026-03-29
**Related:** bean `csl26-jgt4`

## Purpose

Replace scattered, flat `archive`/`archive_location` fields and the arXiv
medium hack with two first-class structs, `ArchiveInfo` and `EprintInfo`.
The goal is to support the broader archival and unpublished domain that major
style guides already cover: letters, diaries, institutional records, oral
histories, photographs, audio-visual items, ephemera, digital drafts, private
collections, and repository-hosted preprints. This resolves the existing
semantic collision between shelfmark and repository place, provides structured
archival hierarchy fields that styles can reorder per their formatting rules,
and defines a cleaner model for preprint identifiers.

## Scope

**In scope:**
- `ArchiveInfo` as generic holding/provenance metadata for archived or privately
  held material
- `EprintInfo` for preprint-server identifiers (on all reference classes)
- `MonographType::Preprint` as a policy exception for standalone preprints
- Expanding `archive` and `eprint` coverage from `Monograph`-only to
  `CollectionComponent` and `SerialComponent`
- New `SimpleVariable` variants for template authoring
- Engine variable resolution for the style-facing archive/eprint fields
- Corrected Chicago archival ordering for the `manuscript:` type-variant
- `archive-place` and `archive-url` as explicit rendering variables

**Out of scope:**
- Legal citation types as a separate feature
- Thesis/dissertation as a distinct top-level type
- Dataset, AV, image, or artifact top-level types
- Automatic repository discovery or registry integration
- Heuristic migration of legacy `archive_location` values into `archive.place`

## Background

### Broad Unpublished and Archival Coverage

Chicago, MLA, APA, and adjacent archival guidance treat unpublished and
archival material as a broad domain rather than a manuscript-only corner case.
Common examples include:

- personal letters, postcards, and correspondence
- diaries, journals, and notebooks
- unpublished manuscripts and student papers
- archival files and folders within named collections
- institutional records, minutes, and internal reports
- oral histories and interview transcripts
- photographs, negatives, slides, and contact sheets
- maps, plans, ephemera, scrapbooks, and albums
- audio/video recordings and born-digital drafts
- private collections and archived organizational collections

Across these categories, the recurring descriptive pieces are stable: creator,
item title or description, date, material type, collection/series, repository,
repository place, and container or shelfmark details. Material type descriptors
(letter, photograph, oral history, etc.) are carried by the existing `genre`
field, which is already present on all reference classes and rendered by styles
(e.g., APA's bracket descriptor `[Letter]`). This spec does not add a
material-type field to `ArchiveInfo`.

### The `archive_location` Semantic Collision

CSL 1.0 uses `archive_location` for two incompatible purposes depending on the
style: (a) the shelfmark / box-folder reference (`"Box 14, Folder 3"`) and
(b) the geographic location of the repository (`"Chicago, IL"`). Citum
inherited this ambiguity. Major archival styles routinely need both values in
the same reference.

### The arXiv Medium Hack

`citum_migrate` and several styles use `article-journal` + `medium: arXiv` as a
proxy for preprints. This conflates publication stage with reference type and
makes server-native identifiers such as `arXiv:2301.00001 [cs.AI]` difficult to
render through first-class variables.

### Prior Art

**biblatex** (preferred prior art per project policy):
- Uses `eprint` / `eprinttype` / `eprintclass` fields across all entry types,
  which is the direct model for `EprintInfo`.
- Often models archival holdings through combinations of `library`, `location`,
  `addendum`, and collection-oriented fields rather than a single flat string.

**CSL community RFC (discourse.citationstyles.org/t/1931):**
- Requests `archive-url` as a stable preservation URL distinct from the primary
  access URL.
- Requests `archive-place` as an explicit geographic field.

**Chicago 17th/18th edition:**
- Common archival order is item title or description, collection, shelfmark or
  container, repository, then city. Major style guides prescribe specific
  formatting rules for container/folder/item components, which motivates
  structured rather than free-form representation.

**APA 7th edition:**
- Archival examples center repository name plus URL, with item format and other
  holding details added as needed.

**MLA 9th edition:**
- Treats archive/repository data as a generic source-side grouping rather than
  a manuscript-only special case.

**Archival description standards (ISAD(G), DACS, EAD):**
- Standard hierarchy: Repository > Collection > Series > Box > Folder >
  Item. The structured fields in `ArchiveInfo` mirror this hierarchy to allow
  style-driven reordering and formatting.

## Design

### ArchiveInfo Struct

`ArchiveInfo` represents generic holding or provenance metadata for items held
in a physical archive, digital repository, private collection, or comparable
custodial context. It is not manuscript-specific.

```rust
/// Holding and provenance metadata for archived or privately held material.
pub struct ArchiveInfo {
    /// Name of the holding repository or archive
    /// (e.g. "Newberry Library", "National Archives"). For private holdings,
    /// use the owner's name as the repository
    /// (e.g. "Private collection of Maria Ortiz").
    /// Uses `MultilingualString` for i18n consistency with `SimpleName.name`
    /// and `Title` — archive names may need transliteration or translation
    /// (e.g. 国立国会図書館 / National Diet Library).
    pub name: Option<MultilingualString>,

    /// Geographic place of the holding repository
    /// (recommended format: "City, Region/Country").
    /// Stored as opaque display text; v1 does not parse or normalize it.
    pub place: Option<String>,

    /// Human-readable named collection or record group
    /// (e.g. "Papers of Carl Sandburg").
    pub collection: Option<String>,

    /// Optional machine-separable collection identifier such as an accession
    /// number or call number for the collection as a whole.
    pub collection_id: Option<String>,

    /// Named series or sub-collection within the collection
    /// (e.g. "Correspondence Series", "Administrative Records").
    pub series: Option<String>,

    /// Box designation within the collection. `box` is a Rust reserved
    /// keyword; the field uses raw identifier syntax `r#box`, which serde
    /// serializes as `box` transparently. Non-box containers (volumes,
    /// cartons) should use the `location` override instead.
    pub r#box: Option<String>,

    /// Folder designation within a container.
    pub folder: Option<String>,

    /// Item, file, or reference-code designation inside the archival
    /// hierarchy.
    pub item: Option<String>,

    /// Free-form display override for the full shelfmark or container string
    /// (e.g. "MS Bodl. Or. 579, fol. 23r"). When present, styles render this
    /// value directly instead of assembling from structured fields.
    /// Used as a legacy-migration target and as an escape hatch for complex
    /// shelfmarks that do not decompose cleanly.
    pub location: Option<String>,

    /// Stable preservation URL for a repository-hosted surrogate or landing
    /// page. This is distinct from the reference's primary `url`.
    pub url: Option<Url>,
}
```

#### Field Precedence

The structured hierarchy fields (`collection_id`, `series`, `box`, `folder`,
`item`) are the **canonical** representation when present. Major style guides
(Chicago, MLA) prescribe specific ordering and formatting rules for these
components (e.g., abbreviation conventions, punctuation between box and
folder). Structured fields allow the engine to assemble and reorder them per
style rules, including locale-aware labels (e.g., "Box 12" in English,
"Boîte 12" in French). Values should be bare identifiers (e.g., `"12"`)
rather than display strings (e.g., `"Box 12"`).

`location` is a **display override / legacy fallback**:

- When both `location` and structured fields are present, `location` takes
  precedence for rendering (the author has manually composed the string).
- When only structured fields are present and `location` is absent, the engine
  assembles a locale-aware display string per style rules (assembly rules
  deferred to implementation).
- When only `location` is present (e.g., migrated legacy data), styles render
  it directly.
- `place` is always a separate geographic field, never part of the shelfmark.
- `url` is a preservation or repository URL. v1 does not define automatic
  precedence between the reference-level `url` and `archive-info.url`; styles that
  want the archival link must render `archive-url` explicitly.

### Archival Examples

Structured archival manuscript example:

```yaml
class: monograph
type: manuscript
title: "Letter from Margaret Fuller to Ralph Waldo Emerson"
issued: 1846-05-11
genre: letter
archive-info:
  name: Houghton Library
  place: "Cambridge, MA"
  collection: Emerson Family Papers
  collection-id: MS Am 1280
  series: Correspondence
  box: "12"
  folder: "4"
  item: "7"
```

Non-manuscript archival example:

```yaml
class: monograph
type: document
title: "Mill Yard Workers at Noon"
issued: 1937
genre: photograph
archive-info:
  name: Wisconsin Historical Society
  place: "Madison, WI"
  collection: Industrial Photography Collection
  collection-id: PH 4021
  box: "3"
  folder: "12"
  item: "Photo 18"
```

Legacy-migration example with `location` override:

```yaml
class: monograph
type: manuscript
title: "Commonplace Book"
issued: "1720~"
genre: manuscript
archive-info:
  name: Bodleian Library
  place: "Oxford, UK"
  location: "MS Bodl. Or. 579, fol. 23r"
```

These examples are illustrative rather than exhaustive. The same model should
be usable for diaries, oral histories, audio recordings, ephemera, digital
drafts, and private collections.

### Design Rationale: Relationship to Contributor

`ArchiveInfo.name` + `ArchiveInfo.place` is structurally parallel to
`SimpleName.name` + `SimpleName.location` (used by `publisher` and other
`Contributor` fields). Both pair an institutional name with a geographic
place. `ArchiveInfo.name` uses `MultilingualString` for the same i18n
reasons `SimpleName.name` does.

`ArchiveInfo` is a dedicated struct rather than a reuse of `Contributor`
because:

- Archive needs hierarchy fields (`collection`, `series`, `box`, `folder`,
  `item`) with no equivalent in the contributor model.
- `Contributor` supports `StructuredName` (given/family) and
  `MultilingualName` variants designed for personal names, which do not apply
  to institutional archive names.
- Nesting a `Contributor` inside `ArchiveInfo` for just name + place would add
  YAML depth without benefit.

The geographic-place concept (`place` on `ArchiveInfo`, `location` on
`SimpleName`) could be unified into a shared type in a future cross-cutting
refactor, but extracting a newtype for what is currently `Option<String>` in
both cases is premature.

### EprintInfo Struct

```rust
/// Preprint-server identifier following the biblatex eprint model.
pub struct EprintInfo {
    /// Server-specific identifier (e.g. "2301.00001", "3556000").
    pub id: String,

    /// Preprint server name in canonical lowercase form
    /// (e.g. "arxiv", "ssrn", "biorxiv").
    /// Producers MAY supply mixed-case values such as "arXiv" or "SSRN";
    /// implementations MUST treat this field as case-insensitive and compare
    /// or normalize on the lowercase form.
    pub server: String,

    /// Optional subject-area classification or server-specific class
    /// (e.g. "cs.AI" for arXiv).
    pub class: Option<String>,
}
```

`EprintInfo` intentionally stays minimal in this draft. Server-specific display
prefix tables or item-level override fields are deferred until real style
requirements justify them.

Preprint example:

```yaml
class: monograph
type: preprint
title: "Robust Planning with Archive-Aware Citation Models"
issued: 2026-02
url: "https://arxiv.org/abs/2602.01234"
eprint:
  server: arxiv
  id: "2602.01234"
  class: cs.DL
```

### MonographType::Preprint

This draft continues to propose a new `MonographType::Preprint` variant for
standalone preprints.

Rationale:

- A preprint's relationship to its server (arXiv, SSRN) is **custodial** — the
  server hosts and preserves the work — not **editorial** (no peer review, no
  volume/issue assignment). This is structurally analogous to how an archived
  manuscript relates to its repository.
- The `Monograph` class with `eprint` metadata models this accurately, whereas
  `SerialComponent` implies an editorial parent-child relationship that does
  not exist at citation time.
- Treating preprints as `article-journal` plus `medium` leaks migration-era
  workarounds into the core model.

Note: the `eprint` *field* is available on all reference classes (see Schema
Placement below) for the common case of published articles that also carry a
preprint identifier. `MonographType::Preprint` handles the distinct case of a
standalone work that has no journal parent.

This is a deliberate policy exception relative to current guidance, not an
accidental inconsistency. The follow-up implementation work must either:

- adopt `MonographType::Preprint` and later reconcile the policy docs, or
- revise the policy and model together before code lands.

### Policy Note

[`../policies/TYPE_ADDITION_POLICY.md`](../policies/TYPE_ADDITION_POLICY.md)
currently preserves the older recommendation to model preprints as
`SerialComponent` with archive metadata. That document is not changed by this
spec revision. The mismatch is intentional for now and must be reconciled
before implementation is treated as complete.

### Schema Placement

`archive_info: Option<ArchiveInfo>` is added to `Monograph`, `CollectionComponent`,
and `SerialComponent`, and serialized as `archive-info`:

`eprint: Option<EprintInfo>` is added to the same three structs. This follows
the biblatex precedent of placing `eprint` on all entry types: journal articles
routinely carry arXiv identifiers, conference papers are deposited on arXiv
pre-conference, and review articles may have SSRN deposits. Restricting
`eprint` to `Monograph` alone would force a future schema break for an
extremely common use case.

This initial attachment surface covers the current source kinds that already
carry archive-like metadata in Citum. Both structs should be documented as
reusable beyond these initial placements.

### SimpleVariable Additions

New variants for template use:

| Variable name          | Resolves to            |
|------------------------|------------------------|
| `archive-name`         | `archive-info.name`          |
| `archive-location`     | `archive-info.location`      |
| `archive-place`        | `archive-info.place`         |
| `archive-collection`   | `archive-info.collection`    |
| `archive-collection-id`| `archive-info.collection-id` |
| `archive-series`       | `archive-info.series`        |
| `archive-box`          | `archive-info.box`           |
| `archive-folder`       | `archive-info.folder`        |
| `archive-item`         | `archive-info.item`          |
| `archive-url`          | `archive-info.url`           |
| `eprint-id`            | `eprint.id`            |
| `eprint-server`        | `eprint.server`        |
| `eprint-class`         | `eprint.class`         |

Because major style guides prescribe distinct formatting rules for box,
folder, and item components, the structured hierarchy fields receive template
variables in this revision. This allows styles to assemble and order these
components according to their own conventions.

### InputReference Accessors

Rather than exposing raw nested structs to the engine, `InputReference` should
provide flat accessors for the style-facing fields:

```rust
fn archive_name(&self) -> Option<&str>
fn archive_location(&self) -> Option<&str>
fn archive_place(&self) -> Option<&str>
fn archive_collection(&self) -> Option<&str>
fn archive_collection_id(&self) -> Option<&str>
fn archive_series(&self) -> Option<&str>
fn archive_box(&self) -> Option<&str>
fn archive_folder(&self) -> Option<&str>
fn archive_item(&self) -> Option<&str>
fn archive_url(&self) -> Option<&Url>
fn eprint_id(&self) -> Option<&str>
fn eprint_server(&self) -> Option<&str>
fn eprint_class(&self) -> Option<&str>
```

Each method matches `Monograph`, `CollectionComponent`, and `SerialComponent`
arms (where the fields exist on the variant); all other arms return `None`.

### Chicago Rendering Fix

The `manuscript:` type-variant block in
`styles/preset-bases/chicago-notes-18th.yaml` (and its sibling
`styles/preset-bases/chicago-author-date-18th.yaml`) should render archival
components in this order, following Chicago 17th/18th archival citation rules:

1. `archive-collection`
2. `archive-location` (or assembled structured fields)
3. `archive-name`
4. `archive-place`

This aligns the current design target with Chicago-style archival ordering.

### Preprint Rendering Direction

If the preprint variant is implemented as proposed here, the initial rendering
target remains:

- Chicago notes: title + server-native identifier + optional class + date
- APA: author, date, title, `[Preprint]`, server, and primary `url`

The exact server-specific display rules stay intentionally narrow in this spec:

- `arXiv:2301.00001 [cs.AI]`
- `SSRN 3556000`

Richer formatting tables are deferred.

## Migration Notes

### Existing Flat Data

Legacy references using flat archive strings remain the migration baseline:

```yaml
archive: "Some Library"
archive_location: "Box 14, Folder 3"
```

Default migration target:

- legacy `archive` -> `archive-info.name`
- legacy `archive_location` -> `archive-info.location`

This spec intentionally does not introduce heuristic remapping from legacy
`archive_location` values into `archive.place`. If such heuristics are ever
added, they must be opt-in.

### Style Migration Direction

Shipped styles that currently use `archive` or `archive-location` should move
toward:

- `archive` -> `archive-name`
- `archive-location` -> shelfmark/container semantics only
- `archive-place` -> repository place when the style needs it

## Known Limitations

**Genre localization.** Material type descriptors (`genre: letter`,
`genre: photograph`) are currently `Option<String>` — a free-form value that
the engine cannot localize. A style rendering `[Letter]` in English cannot
produce `[Lettre]` in French without a formalized genre vocabulary with
locale-keyed labels. This is a cross-cutting limitation affecting all
reference types, not just archival material. Genre formalization is tracked
as a separate follow-up (see bean `csl26-ldgf`); archival material-type
rendering will depend on that work for proper localization.

## Deferred Design

The following are intentionally not decided in this revision:

- engine assembly rules for composing a display string from structured
  hierarchy fields when `location` is absent
- virtual grouping variables such as `archive-source` or `archive-shelfmark`
- heuristic migration from legacy `archive_location` to `archive.place`
- `EprintInfo.display_prefix`
- server-specific formatting tables beyond the simple examples above

## Acceptance Criteria

- [ ] The spec defines `ArchiveInfo` as generic archival/provenance metadata,
      not manuscript-only metadata.
- [ ] The spec separates repository place from shelfmark/container semantics.
- [ ] Structured hierarchy fields (`collection_id`, `series`, `box`,
      `folder`, `item`) are the canonical representation; `location` is
      documented as a display override / legacy fallback.
- [ ] The spec documents `archive-info.url` as a distinct preservation URL and
      states that v1 styles must request it explicitly.
- [ ] The spec includes at least one structured archival example with
      repository, collection, series, box, folder, and item details.
- [ ] The spec includes at least one non-manuscript archival example.
- [ ] The spec includes a legacy-migration example using `location` as
      fallback.
- [ ] `EprintInfo` is placed on all three reference classes, following
      biblatex precedent.
- [ ] The spec keeps `EprintInfo` minimal and defers display-prefix expansion.
- [ ] The spec explicitly acknowledges the current policy mismatch around
      preprint modeling.
- [ ] `genre` is documented as the existing mechanism for material-type
      descriptors.

When this spec moves to Active, implementation-level acceptance criteria will
be added covering: struct definitions in citum-schema-data, InputReference
accessors, engine variable resolution, style updates, migration mapping, JSON
schema regeneration, and integration tests.

## Changelog

- 2026-03-29: PR review pass. Dropped spec version number. Changed
  `ArchiveInfo.name` to `MultilingualString` for i18n consistency with
  `SimpleName`/`Title`. Added design rationale for Contributor parallel.
  Fixed `r#box` doc comment. Added full Chicago style file paths. Documented
  genre localization limitation as a known gap.
- 2026-03-28: Reverted `container` back to `box`. Structured field
  values are bare identifiers (e.g., `"12"`), so the field name carries the
  semantic label for locale-aware rendering. Rust keyword handled via
  `#[serde(rename = "box")]`. Non-box containers use `location` override.
- 2026-03-28: Architectural review pass. Renamed `box` to `container`
  (Rust reserved keyword; aligns with DACS/ISAD(G) terminology). Added `series`
  to the hierarchy. Made structured fields canonical with `location` as display
  override. Expanded `eprint` placement to all three reference classes
  (biblatex precedent). Added template variables for all hierarchy fields.
  Strengthened preprint rationale (custodial vs editorial framing). Clarified
  `name` as repository/archive name. Documented `genre` as material-type
  mechanism. Added legacy-migration example. Added Chicago edition
  clarification.
- 2026-03-28: Broadened the archival model, added structured hierarchy
  fields, clarified archive URL/place semantics, and documented the preprint
  policy mismatch without making code decisions final.
- 2026-03-28: Initial draft.
