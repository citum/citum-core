# Type System Refactor v3

**Status:** Active
**Date:** 2026-04-04
**Branch:** `type/refactor-3`

## 1. Motivation

Three concrete problems in `examples/chicago-note-converted.yaml` and
recent schema work motivate this refactor:

1. **Contributor role proliferation.** `Monograph` carries `author`,
   `editor`, `translator`, `recipient`, `interviewer`, and `guest` as
   separate named fields. Each new role requires a new field on every
   affected struct. `recipient` and `interviewer` are only valid for two
   `MonographType` subtypes, yet they are present on the general struct.

2. **Monograph overreach.** Films, interviews, personal communications,
   and recordings are all expressed as `class: monograph` with different
   `type` tags. These are semantically distinct reference classes with
   incompatible contributor role sets and rendering logic.

3. **No work-level reference class.** When citing a film, the citee is
   the work (the film itself), not a specific physical or digital release.
   The current model forces films into `class: serial-component` of a
   `broadcast-program` container — structurally wrong. There is no clean
   way to express a work-level citation.

---

## 2. Design principles

### 2.1 Citation-oriented abstraction distinction

Citum is not adopting FRBR as a system architecture. This refactor uses a
narrower, citation-oriented distinction that is useful for rendering:

- **Work-like citation classes** represent the creative work that is being
  cited as the citee itself. Typical fields are title, contributors, original
  date, language, and abstract genre.

- **Version-like citation classes** represent a specific realized form of
  a work: edition, release, publication form, or container-published
  instance. Most existing Citum classes (`Monograph`,
  `SerialComponent`, etc.) operate this way.

**This distinction is local and pragmatic, not the start of a generalized
bibliographic entity model.** Citum will likely never support full FRBR,
and it does not need to in order to solve citation rendering correctly.

This wave therefore does **not** introduce:

- a Work/Expression/Manifestation/Item graph,
- a generalized bibliographic relationship system,
- or an expectation that every Citum type must be classified through
  full FRBR semantics.

The abstraction level is encoded by the class itself, not by a per-instance
field. A `Monograph` remains a version-like concept. An `AudioVisualWork`
is a work-like concept. Adding an `abstraction-level` field to
`Monograph` would still be a category error.

### 2.2 Composition over inheritance

Rust has no inheritance. Shared structure is expressed via composed
structs. Work-level reference classes compose a shared `WorkCore` struct
that carries the common work-level fields. Future work-level classes
(`Artwork`, `MusicalWork`) compose the same `WorkCore`.

### 2.3 Targeted shorthands for common roles

To maintain authoring ergonomics for the most common citation cases, the
named fields `author`, `editor`, and `translator` remain valid YAML
input. They deserialize into the canonical `contributors` list via
concrete `*Deser` deserialization shims.

In a targeted breaking change, niche legacy fields (`recipient`,
`interviewer`, `guest`) are removed as top-level shorthands and must be
authored using the unified `contributors` array.

Serialization uses `contributors` as the canonical form.

### 2.4 Hybrid-model fit for `AudioVisualWork`

`AudioVisualWork` is a narrowly-scoped semantic class for cases where the
citee is the work itself and the existing structural classes distort that
meaning. This is not a precedent for flattening all structural classes.

It remains consistent with Citum's hybrid type model because:

- structural classes are still preferred where parent-child publication
  structure is load-bearing;
- `AudioVisualWork` is introduced only where the current structural fit is
  semantically wrong for common citation cases;
- the proposal does not imply a broader move away from structural classes
  for articles, chapters, or similar publication components.

Within the active type-addition policy, `AudioVisualWork` should be read as
a bounded citation-model addition rather than as a general invitation to
split all subtypes into flat semantic classes.

---

## 3. Changes

### 3.1 Unified contributors model

**Problem:** Named contributor fields proliferate per role, are
class-specific, and cannot be extended without schema changes.

**Solution:** Add `contributors: Vec<ContributorEntry>` to all reference structs.
Each entry pairs a `role` with a `contributor`.

```rust
/// A contributor role for use in the unified contributors list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ContributorRole {
    Author,
    Editor,
    Translator,
    Director,
    Performer,
    Composer,
    Illustrator,
    Narrator,
    Host,
    Guest,
    Interviewer,
    Recipient,
    Compiler,
    Producer,
    Writer,
    /// An open extension point for domain-specific roles.
    #[serde(untagged)]
    Custom(String),
}

/// A single entry in a reference's contributors list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub struct ContributorEntry {
    /// The role this contributor plays in relation to the work.
    pub role: ContributorRole,
    /// The contributor (name, organization, or list).
    pub contributor: Contributor,
}
```

**Canonical form and compatibility rules:**

- `contributors` is the only canonical serialized form after this refactor.
- Ubiquitous named fields in YAML (`author:`, `editor:`, `translator:`)
  remain valid input shorthands for ergonomics and are retained only on
  `XDeser` structs.
- Niche legacy fields (`recipient`, `interviewer`, `guest`) are removed;
  existing data using these must migrate to explicit `contributors` entries.
- Normalization starts with explicit `contributors` entries in source order,
  then folds supported legacy named fields after them in a fixed role order.
- Multiple entries with the same role remain valid; the model does not
  collapse same-role contributors into a single slot.
- Legacy named fields do not append an entry when an equivalent
  `ContributorEntry` already exists for the same role and contributor value.
  *(Note: This relies on `PartialEq` for `Contributor`. Ensure the
  equality check is robust enough to handle slight variations in
  contributor data, such as literal vs. parsed name fields, to prevent
  duplicate entries.)*
- Existing template variables and accessor methods (`author`, `editor`,
  `translator`, `recipient`, `interviewer`, `guest`) remain supported in
  this wave; the engine maps the `contributors` array back into these
  variables by role.

The `Monograph` struct gains a `contributors` field; the named fields are
retained only in `MonographDeser` as deserialization shorthands.

```rust
// In MonographDeser → Monograph normalization:
fn fold_named_into_contributors(raw: &MonographDeser) -> Vec<ContributorEntry> {
    let mut contributors = raw.contributors.clone().unwrap_or_default();
    // Preserve explicit contributors order; append legacy shorthands only when
    // the same role+contributor value is not already present.
    for (role, contributor) in [
        (ContributorRole::Author,      &raw.author),
        (ContributorRole::Editor,      &raw.editor),
        (ContributorRole::Translator,  &raw.translator),
    ] {
        if let Some(c) = contributor {
            if !contributors.iter().any(|e| {
                e.role == role && e.contributor == *c
            }) {
                contributors.push(ContributorEntry { role, contributor: c.clone() });
            }
        }
    }
    contributors
}
```

**Engine and template compatibility:** Existing template variables remain
named (`author`, `editor`, `translator`, `recipient`, `interviewer`,
`guest`). The binding layer maps `contributors` back into those names by role.
This preserves the current engine contract while consolidating storage.

| Existing binding/accessor | Contributor role |
|---------------------------|------------------|
| `author` | `Author` |
| `editor` | `Editor` |
| `translator` | `Translator` |
| `recipient` | `Recipient` |
| `interviewer` | `Interviewer` |
| `guest` | `Guest` |

**Engine accessors** are updated to read from `contributors` by role:

```rust
impl Monograph {
    pub fn authors(&self) -> impl Iterator<Item = &Contributor> {
        self.contributors.iter()
            .filter(|e| e.role == ContributorRole::Author)
            .map(|e| &e.contributor)
    }
    pub fn editors(&self) -> impl Iterator<Item = &Contributor> { /* … */ }
    pub fn by_role(&self, role: &ContributorRole) -> impl Iterator<Item = &Contributor> {
        self.contributors.iter()
            .filter(move |e| &e.role == role)
            .map(|e| &e.contributor)
    }
}
```

This preserves the existing engine API surface while the internal model
consolidates. *(Note: While iterating over the `contributors` list for each
role accessor introduces a slight `O(N)` overhead on the rendering hot
path compared to direct field access, lists are typically small enough
that this is negligible. This should be monitored.)*

**Generic contributor accessor on `InputReference`:** In addition to the
role-specific accessors above, `InputReference` gains a single generic
accessor:

```rust
impl InputReference {
    /// Returns contributors matching `role` for any reference class that
    /// carries a contributors list.
    pub fn contributor(
        &self,
        role: ContributorRole,
    ) -> Option<Contributor> { … }
}
```

This replaces the former pattern of adding one typed method per role
(`guest()`, `interviewer()`, `recipient()`). Those methods are removed;
callers use `contributor(ContributorRole::Guest)` etc. instead.
`contributor(role)` is the mechanism for explicit role lookups in
type-specific template contexts (listing the interviewer alongside the
guest, crediting the composer separately from the performer, etc.).
When multiple contributors match, the accessor returns a folded
`ContributorList`.

**`author()` on `AudioVisualWork`:** The `author()` accessor is retained
on `InputReference` as a semantic "primary contributor" method. For
`AudioVisual` variants it dispatches by subtype rather than returning
`None`:

| `AudioVisualType` | `author()` resolves to |
|-------------------|------------------------|
| `Film` | `contributor(Director)` |
| `Episode` | `contributor(Director)` |
| `Recording` | `contributor(Composer)` if present, else `contributor(Performer)` |
| `Broadcast` | `None` (no fixed primary) |

The recording fallback order (Composer → Performer) reflects the
distinction between classical works (where the composer is the
bibliographic primary) and popular recordings (where the performer is).
Both roles may coexist; the template can retrieve either via `contributor(role)`
when the default is not appropriate.

### 3.2 Work-level reference class: `AudioVisualWork`

**Problem:** Films, TV episodes, and recordings are forced into
`Monograph` or `SerialComponent`, neither of which is structurally
correct. Many of these citations target the work itself rather than a
specific physical release.

**Solution:** Add `class: audio-visual` as a new `InputReference`
variant. It composes `WorkCore` (see §3.3) and adds the minimum fields
needed to identify an audio-visual work within a series.

```rust
/// An audio-visual work: film, TV episode, recording, or broadcast.
///
/// This is a work-like reference class. It represents the creative work
/// when that work is the citee, not a fully modeled release hierarchy.
/// Rich manifestation details remain out of scope for this wave.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(from = "AudioVisualDeser", rename_all = "kebab-case")]
pub struct AudioVisualWork {
    /// Unique identifier for this reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RefID>,
    /// Subtype for style-directed rendering.
    pub r#type: AudioVisualType,
    /// Shared work-level fields.
    #[serde(flatten)]
    pub core: WorkCore,
    /// Container series or program (work-to-work relation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<WorkRelation>,
    /// Season and episode numbering.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub numbering: Vec<Numbering>,
    /// Production company, studio, or network. Added pragmatically to map
    /// to the publisher field for existing styles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<Publisher>,
    /// Physical delivery format or presentation descriptor (e.g., 'compact disc',
    /// 'in Korean, with English subtitles'). Not for digital distribution hosts —
    /// use `platform` for those.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<String>,
    /// Digital distribution host or streaming service (e.g., 'YouTube', 'Netflix',
    /// 'Spotify'). Distinct from `medium`, which covers physical carriers and
    /// presentation format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// URL or streaming location.
    #[serde(alias = "URL", skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    /// Date accessed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<EdtfString>,
    /// Freeform note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Discriminates audio-visual subtypes for style-directed formatting.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum AudioVisualType {
    /// A film or motion picture (default).
    #[default]
    Film,
    /// A single episode of a TV series, podcast, or web series.
    Episode,
    /// A recording (music video, concert, etc.).
    Recording,
    /// A broadcast program.
    Broadcast,
}
```

**YAML shape** for the Parasite example:

```yaml
class: audio-visual
type: film
title: Parasite
contributors:
  - role: director
    contributor:
      family: Bong
      given: Joon-ho
issued: "2019"
publisher:
  name: Barunson E&A
  place: South Korea
language: ko
genre: feature-film
```

The Brady Bunch episode:

```yaml
class: audio-visual
type: episode
title: "My Fair Opponent"
contributors:
  - role: director
    contributor:
      literal: Peter Baldwin
issued: "1973-01-05"
container:
  title: The Brady Bunch
  class: audio-visual
  type: broadcast
numbering:
  - type: season
    value: "5"
  - type: episode
    value: "5"
```

Note: `season` and `episode` are `NumberingType::Custom` variants in the
current schema. This refactor is a natural point to promote them to
first-class `NumberingType` variants given their prevalence in
audio-visual citations.

### 3.3 WorkCore: shared work-level struct

`WorkCore` carries the fields that are meaningful for work-like citation
classes. It does not carry duration, medium, publisher, edition, or
identifiers like ISBN — those belong to a cited version or release when
they matter.

```rust
/// Shared fields for all work-level reference classes.
///
/// Composed into `AudioVisualWork` and future work-level classes
/// (`Artwork`, `MusicalWork`). Not used directly as an `InputReference`
/// variant — always embedded via `#[serde(flatten)]`.
#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub struct WorkCore {
    /// Title of the work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    /// Optional short form of the title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_title: Option<String>,
    /// Unified contributor list with explicit role tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contributors: Vec<ContributorEntry>,
    /// Original creation or release date of the work.
    #[serde(skip_serializing_if = "EdtfString::is_empty")]
    pub issued: EdtfString,
    /// BCP 47 language of the work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<LangID>,
    /// Abstract genre (e.g., `"feature-film"`, `"documentary"`, `"novel"`).
    /// Not the delivery medium — that is a manifestation attribute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
}
```

**Invariant:** Fields that describe how a work is delivered or packaged
(duration, medium, platform, publisher, distributor, ISBN, edition) MUST
NOT be added to `WorkCore`. They belong on version-like structs or a
future companion model if Citum later needs one.

*(Note: `publisher` and `medium` are added to `AudioVisualWork` as a pragmatic
exception because existing citation styles overwhelmingly rely on them to render
basic audio-visual citations, avoiding the need for a full release model for
common film/broadcast cases.)*

Known edge cases may require some version-specific facts in practical
citation workflows. This spec does **not** introduce a full release or
version companion model. Manifestation-heavy cases may remain on existing
classes until a focused follow-up spec defines that shape.

**YAML example** showing `WorkCore` fields in context via `AudioVisualWork`:

```yaml
class: audio-visual
type: film
# WorkCore fields (flattened):
title: Parasite
contributors:
  - role: director
    contributor:
      family: Bong
      given: Joon-ho
  - role: writer
    contributor:
      family: Bong
      given: Joon-ho
issued: "2019"
language: ko
genre: feature-film
# AudioVisualWork-specific fields:
publisher:
  name: Barunson E&A
url: https://www.imdb.com/title/tt6751668/
```

---

## 4. Scope boundaries

The following are explicitly **not in scope** for this refactor:

| Topic | Rationale |
|-------|-----------|
| Replacing `container` with `is-part-of` | Rename with no rendering benefit |
| Nesting identifiers under `identifiers:` block | Flat `doi`/`isbn`/`issn` fields are simpler and already work |
| `work-example` / `has-part` graph relations | Library catalog features, not citation rendering needs |
| Full FRBR entity modeling | Out of scope for Citum generally, not just this wave |
| `is-based-on` / `original-work` relations | Out of scope; `original` field covers reprints but should be moved to a shared location (not Monograph-only) in a follow-up |
| Full flat type vocabulary | Structural classes are load-bearing for 147 production styles |
| `Artwork` or `MusicalWork` classes | Deferred — add when a concrete style demands it |
| Release/version companion model for audio-visual citations | Defer until a concrete manifestation-heavy style requirement justifies it |
| H.R. legislative bills | Defer pending a dedicated legislation/bill class; current `class: monograph, type: document, genre: H.R.` is defensible |

---

## 5. Files affected

| File | Change |
|------|--------|
| `crates/citum-schema-data/src/reference/contributor.rs` | Add `ContributorRole` and `ContributorEntry` for the unified contributor model |
| `crates/citum-schema-data/src/reference/types/structural.rs` | Add `contributors: Vec<ContributorEntry>` to structural types (`Monograph`, `Collection`, `CollectionComponent`, `SerialComponent`, `Serial`) and retain targeted shorthands via deserialization shims |
| `crates/citum-schema-data/src/reference/types/specialized.rs` | Add `WorkCore`, `AudioVisualWork`, and `AudioVisualType`; carry `contributors` on work-level audio-visual data |
| `crates/citum-schema-data/src/reference/mod.rs` | Add `AudioVisual(Box<AudioVisualWork>)` to `InputReference` and contributor-role accessors over canonical `contributors` |
| `crates/citum-engine/src/…` | Update contributor accessors to read from `contributors` by role |
| `examples/chicago-note-converted.yaml` | Update film and broadcast entries to use `class: audio-visual` |
| `docs/policies/TYPE_ADDITION_POLICY.md` | Document `ContributorRole` extension policy and `AudioVisualWork` rationale |

---

## 6. Acceptance criteria

- [x] The spec states explicitly that Citum is not adopting full FRBR and
      that the work-like versus version-like distinction is citation-oriented.
- [x] `AudioVisualWork` is justified in the spec as a bounded hybrid-model
      addition rather than a precedent for flattening structural classes.
- [x] `contributors` is defined as the canonical serialized form.
- [x] Targeted shorthands (`author`, `editor`, `translator`) are justified
      for ergonomics; niche legacy shorthands are removed.
- [x] Merge order, dedupe behavior, and compatibility with existing named
      template variables/accessors are specified normatively.
- [x] The spec states that manifestation-heavy audio-visual cases may stay
      on existing classes until a focused follow-up model exists.

## 7. Open questions

1. **Generic `Work` base class.** Should `AudioVisualWork` be the first
   specialization of a more generic `class: work` discriminant, or should
   each work-level type be its own top-level `InputReference` variant?
   The composition approach (§3.3) makes both paths viable.
   **Recommendation:** Relying on specific top-level `InputReference`
   variants (like `AudioVisualWork`) is preferable to a generic `class:
   work` unless a concrete style explicitly requires an untyped generic
   work. Composition handles code reuse sufficiently.

2. **`contributors` on legal types.** Legal reference classes (`LegalCase`,
   `Statute`, etc.) have no named contributor fields today. They should
   eventually compose `contributors` too, but legal contributor roles
   (counsel, petitioner, etc.) need separate vocabulary work.
   **Recommendation:** Deferring legal types is a smart scope boundary.
   Legal citations have highly specialized roles that deserve their own
   targeted design phase.

---

## 8. Changelog

- **2026-04-04**: Initial draft sketch (v3).
- **2026-04-04**: Replaced initial sketch with scoped implementation
  spec. Introduced `ContributorRole`, `ContributorEntry`, and
  `AudioVisualWork`.
- **2026-04-04**: Narrowed work model: clarified FRBR non-adoption,
  contributor compatibility rules, and bounded `AudioVisualWork` scope.
- **2026-04-04**: Applied review feedback: added `PartialEq` robustness
  note, engine performance note, and recommendations for open questions.
- **2026-04-04**: Renamed `creators` -> `contributors` for Dublin Core
  alignment. Pruned legacy shorthands to only `author`, `editor`, and
  `translator` (breaking change for `guest`, `recipient`, `interviewer`).
- **2026-04-04**: Added `publisher` and `medium` to `AudioVisualWork` as
  pragmatic exceptions to map to studios and formats in existing styles.
- **2026-04-05**: Added `platform` to `AudioVisualWork` to distinguish
  digital distribution hosts (YouTube, Netflix) from physical carriers
  (`medium`). Added H.R. bills to scope boundary table.
- **2026-04-06**: Resolved contributor accessor design: added generic
  `contributor(role)` on `InputReference`; removed typed role-specific
  methods (`guest()`, `interviewer()`, `recipient()`). Documented
  `author()` type-aware dispatch for `AudioVisualWork` subtypes.
  Reclassified Weingartner example to `class: audio-visual, type:
  recording` with `role: composer`.
