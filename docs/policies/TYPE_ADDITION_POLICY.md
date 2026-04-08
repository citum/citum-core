# Type Addition Policy

**Status:** Active Policy
**Version:** 1.1
**Date:** 2026-03-29
**Related:** TYPE_SYSTEM_ARCHITECTURE.md, ENUM_VOCABULARY_POLICY.md

## Purpose

This policy provides clear criteria for deciding when to add a new top-level reference type to Citum versus using existing structural types (SerialComponent, CollectionComponent, Monograph, Collection).

## Architecture Model: Hybrid

Citum uses a **hybrid type system**:

- **Structural types** for academic references with meaningful parent-child relationships (journal articles, book chapters)
- **Flat types** for references where the container is a locator rather than a semantic parent (legal cases, treaties, datasets)

This balances:
- **Data efficiency** (parent metadata reused across components)
- **Style clarity** (explicit type-specific overrides)
- **User mental models** (semantic type names)

## Prior Art

**biblatex** (flat model):
- 31 entry types as distinct database types, not subtypes
- Relationships via fields (`crossref`, `xref`) not type hierarchy
- Types chosen based on semantic distinction and field schema differences
- Example: `@mvbook` (multi-volume) separate from `@book` because field requirements differ

**CSL 1.0** (flat model):
- 34 types defined as flat enumeration (article-journal, book, legal_case, treaty)
- Container relationships via variables (`container-title`) not parent types
- Types chosen for citation style discrimination

**Citum** (hybrid model):
- Structural types where parent-child relationship provides efficiency (SerialComponent → Serial)
- Flat types where semantic distinction and style clarity outweigh data model elegance

## Decision Criteria: The 4-Factor Test

Add a new top-level type when **ALL** of the following are true:

### 1. Semantic Distinction

**Test:** Do users think of this as a fundamentally different thing?

**Threshold:** The reference has a distinct identity in scholarly discourse (legal case ≠ journal article, dataset ≠ report).

**Examples:**
- ✅ LegalCase: Legal scholars think "court decision" not "serial component"
- ✅ Treaty: International law context, distinct from academic articles
- ❌ BlogPost: Variant of article, users think "online article"
- ❌ ReviewBook: Variant of review, genre field suffices

### 2. Style Discrimination

**Test:** Do major citation styles format this type differently?

**Threshold:** At least 20% of major styles (APA, Chicago, MLA, IEEE, Harvard, Nature, AMA, ACS, AIP, Vancouver) require distinct formatting.

**Evaluation:**
- Check style manuals for dedicated sections
- Look for different field ordering, punctuation, emphasis
- Consider legal/domain-specific style guides (Bluebook, AMA)

**Examples:**
- ✅ LegalCase: Legal styles (Bluebook, ALWD) have dedicated case citation rules
- ✅ Statute: Legislative citation formats differ from article citations
- ❌ MagazineArticle: Minor differences from journal articles (often just volume/issue suppression)
- ❓ Dataset: Emerging (DataCite styles exist, APA 7th has data citation guidelines)

### 3. Field Schema Difference

**Test:** Do required/expected fields differ significantly from existing types?

**Threshold:** At least 3 fields that are:
- Required for this type but not others, OR
- Expected in this type but rare/nonsensical in others

**Examples:**
- ✅ LegalCase: `authority` (court), `reporter`, `docket-number` (unique to legal)
- ✅ Dataset: `size`, `format`, `version`, `repository` (unique to datasets)
- ❌ Preprint: Same fields as article, just different publication stage
- ❌ ReviewBook: Same fields as article/review, genre distinguishes

### 4. No Meaningful Parent

**Test:** Is the "container" a locator rather than a semantic parent?

**Threshold:** If the container requires independent citation OR multiple containers are valid (parallel citations), it's a locator not a parent.

**Examples:**
- ✅ LegalCase: Reporter is where to find it, not what it is (parallel citations in multiple reporters)
- ✅ Treaty: Treaty series is publication venue, not semantic container
- ❌ JournalArticle: Journal IS the semantic container (article is part of journal)
- ❌ Chapter: Book IS the semantic container (chapter is part of book)
- ❓ Preprint: arXiv is locator (published version is in journal), but single-container

## Decision Flowchart

```
┌─────────────────────────────────────────────────┐
│ Does this reference require different           │
│ citation formatting across major styles?        │
│ (Factor 2: Style Discrimination)                │
└─────────┬───────────────────────────────────────┘
          │
    ┌─────┴─────┐
    │    NO     │──► Use existing type + optional field
    └───────────┘    (e.g., genre, medium)
          │
          │ YES
          ▼
┌─────────────────────────────────────────────────┐
│ Do users think of this as fundamentally         │
│ different from existing types?                  │
│ (Factor 1: Semantic Distinction)                │
└─────────┬───────────────────────────────────────┘
          │
    ┌─────┴─────┐
    │    NO     │──► Use subtype or genre field
    └───────────┘
          │
          │ YES
          ▼
┌─────────────────────────────────────────────────┐
│ Does it have unique required/expected fields?   │
│ (Factor 3: Field Schema Difference)             │
└─────────┬───────────────────────────────────────┘
          │
    ┌─────┴─────┐
    │    NO     │──► Use existing type + optional fields
    └───────────┘
          │
          │ YES
          ▼
┌─────────────────────────────────────────────────┐
│ Is the "container" a locator rather than        │
│ a semantic parent?                              │
│ (Factor 4: No Meaningful Parent)                │
└─────────┬───────────────────────────────────────┘
          │
    ┌─────┴─────┐
    │    NO     │──► Use structural type with parent
    └───────────┘    (SerialComponent, CollectionComponent)
          │
          │ YES
          ▼
┌─────────────────────────────────────────────────┐
│ ADD NEW TOP-LEVEL TYPE                          │
│                                                 │
│ Example: LegalCase, Treaty, Dataset             │
└─────────────────────────────────────────────────┘
```

## Examples: Applying the 4-Factor Test

### LegalCase (Court Decision)

| Factor | Score | Rationale |
|--------|-------|-----------|
| 1. Semantic | ✅ | Legal scholars think "case" not "article" |
| 2. Style | ✅ | Bluebook, ALWD have dedicated case citation rules |
| 3. Schema | ✅ | `authority` (court), `reporter`, `docket-number` unique |
| 4. No Parent | ✅ | Reporter is locator (parallel citations common) |

**Decision:** Add flat type `LegalCase`

### Treaty (International Agreement)

| Factor | Score | Rationale |
|--------|-------|-----------|
| 1. Semantic | ✅ | Distinct identity in international law |
| 2. Style | ✅ | Legal and international relations styles differ |
| 3. Schema | ✅ | Treaty-specific fields (parties, ratification date) |
| 4. No Parent | ✅ | Treaty series is publication venue, not semantic parent |

**Decision:** Add flat type `Treaty`

### BlogPost (Blog Article)

| Factor | Score | Rationale |
|--------|-------|-----------|
| 1. Semantic | ❌ | Users think "online article" |
| 2. Style | ❌ | Cited same as magazine/newspaper articles |
| 3. Schema | ❌ | Same fields as article (title, author, date, URL) |
| 4. No Parent | ❌ | Blog IS the parent (like magazine/journal) |

**Decision:** Use `SerialComponent` with parent `Serial(type: Blog)`

### Dataset (Research Data)

| Factor | Score | Rationale |
|--------|-------|-----------|
| 1. Semantic | ✅ | Distinct scholarly artifact (not a publication) |
| 2. Style | ✅? | DataCite, APA 7th have data citation guidelines |
| 3. Schema | ✅ | `size`, `format`, `version`, `repository` unique |
| 4. No Parent | ✅ | Repository is locator (Zenodo, figshare, Dryad) |

**Decision:** Add flat type `Dataset` (emerging, validate against APA/Chicago/Nature)

### MagazineArticle (Popular Press)

| Factor | Score | Rationale |
|--------|-------|-----------|
| 1. Semantic | ❌ | Variant of article |
| 2. Style | ⚠️ | Minor differences (volume/issue suppression) |
| 3. Schema | ❌ | Same fields as journal article |
| 4. No Parent | ❌ | Magazine IS the parent |

**Decision:** Use `SerialComponent` with parent `Serial(type: Magazine)`

**Rationale for structural type:** Style discrimination (Factor 2) is minor and achievable via parent-type overrides. Data efficiency (journal metadata reused) outweighs template complexity.

### Preprint (Pre-publication Article)

| Factor | Score | Rationale |
|--------|-------|-----------|
| 1. Semantic | ⚠️ | Variant of article (publication stage, not type) |
| 2. Style | ⚠️ | Minor differences (archive identifier instead of DOI) |
| 3. Schema | ❌ | Same fields as article + archive identifier |
| 4. No Parent | ⚠️ | arXiv is locator, but published version is in journal |

**Decision:** **Ambiguous** - Use `SerialComponent` with `archive` field for now. Monitor style evolution.

**Rationale:** Preprints are temporally-qualified articles (pre-peer-review). Most citation styles treat them as articles with archive metadata. However, if dedicated preprint citation formats emerge, re-evaluate.

## Migration Compatibility Factor (Optional 5th Factor)

**Test:** If CSL 1.0 has a distinct type for this, does style discrimination justify a Citum flat type or is a structural subtype sufficient?

**Purpose:** Guide CSL 1.0 → Citum migration decisions.

**Examples:**

**CSL 1.0 Types → Citum Decision:**

| CSL 1.0 Type | Citum Type | Rationale |
|--------------|-----------|-----------|
| `article-journal` | SerialComponent(parent: AcademicJournal) | Parent reuse efficiency |
| `article-magazine` | SerialComponent(parent: Magazine) | Same structure, parent differs |
| `article-newspaper` | SerialComponent(parent: Newspaper) | Same structure, parent differs |
| `legal_case` | LegalCase | Passes 4-factor test |
| `treaty` | Treaty | Passes 4-factor test |
| `book` | Monograph(type: Book) | Monolithic work |
| `chapter` | CollectionComponent | Parent-child relationship |

**Trade-off:** Accept template complexity (parent-type discrimination) for data model efficiency when factors 3-4 fail.

## Policy Enforcement

**For new type proposals:**

1. **Create GitHub issue** using "New Reference Type" template
2. **Complete 4-factor test** with evidence for each factor
3. **Provide examples** from at least 3 major citation styles
4. **List unique fields** required/expected for this type
5. **Discuss parent relationship** - locator vs semantic container?

**Review criteria:**

- All 4 factors must be ✅ for flat type
- If any factor is ❌ or ⚠️, justify why exception is warranted
- Consider CSL 1.0 compatibility (optional 5th factor)
- Prefer structural types when factors 3-4 fail (efficiency > template simplicity)

## Rationale for Hybrid Model

**Why not pure flat (Option C)?**

1. **Data bloat:** Repeating journal metadata (title, ISSN, publisher) across 50 articles from the same journal wastes space
2. **Update complexity:** If journal name changes, must update 50 records vs 1 parent record
3. **Query inefficiency:** "All articles from APSR" requires full scan vs parent ID lookup

**Why not pure structural (Option B)?**

1. **Violates "explicit over magic":** Parent type discrimination (`serial-component.parent-type.treaty-series`) is procedural logic in declarative templates
2. **User confusion:** Legal experts think "legal case" not "serial component of reporter series"
3. **Parallel citations:** Same treaty in multiple reporters breaks parent-child model

**Hybrid model achieves:**

- ✅ Data efficiency where parent-child is meaningful (academic references)
- ✅ Style clarity where semantic distinction matters (legal, datasets)
- ✅ Alignment with user mental models (legal case, treaty, dataset)
- ✅ CSL 1.0 compatibility (maps flat types 1:1, structural types many:1)

## Audit of Current Types

**Structural types (parent-child relationship):**

| Citum Type | Rationale |
|---|---|
| SerialComponent(Article) | ✅ Journal is semantic parent, metadata reused |
| SerialComponent(Post) | ✅ Blog/Magazine is parent |
| SerialComponent(Review) | ✅ Journal is parent |
| CollectionComponent(Chapter) | ✅ Book/edited volume is semantic parent |
| CollectionComponent(Document) | ✅ Conference proceedings or collection is parent |
| Collection(Anthology) | ✅ Edited collection of independent works |
| Collection(Proceedings) | ✅ Conference proceedings container |
| Collection(EditedBook) | ✅ Edited book container |
| Collection(EditedVolume) | ✅ Edited volume container |

**Flat types (no parent or locator parent):**

| Citum Type | Rationale |
|---|---|
| LegalCase | ✅ Passes 4-factor test |
| Statute | ✅ Passes 4-factor test |
| Treaty | ✅ Passes 4-factor test |
| Hearing | ✅ Legislative context, unique fields |
| Regulation | ✅ Regulatory context, unique fields |
| Brief | ✅ Legal filing context |
| Classic | ✅ Standard citation forms (Aristotle, Bible) |
| Patent | ✅ Distinct identity, unique fields (patent-number, jurisdiction) |
| Dataset | ✅ Distinct artifact, unique fields (format, size, repository) |
| Standard | ✅ Distinct artifact, unique fields (standard-number, status) |
| Software | ✅ Distinct artifact, unique fields (version, repository, license) |
| Monograph(Book) | ✅ Monolithic work, no parent |
| Monograph(Report) | ✅ Monolithic work |
| Monograph(Thesis) | ✅ Academic work, institution is not parent |
| Monograph(Webpage) | ✅ Web content, site is not semantic parent |
| Monograph(Interview) | ✅ Standalone interview source |
| Monograph(Manuscript) | ✅ Unpublished/archival document |
| Monograph(Preprint) | ✅ Preprint server is custodial, not editorial |
| Monograph(PersonalCommunication) | ✅ Letters, emails; no meaningful parent |
| Monograph(Document) | ✅ Generic standalone document |

**Policy compliance:** All current types conform to the 4-factor test or structural efficiency rationale.

## Future Type Candidates

Monitor these reference types for potential addition:

| Type | Status | Factors | Action |
|---|---|---|---|
| Performance | Low | ✅⚠️✅✅ | Music/theater domain, niche styles |
| Artwork | Low | ✅⚠️✅✅ | Art history domain, niche styles |
| Map | Low | ✅⚠️✅✅ | Cartographic works, geographic/historical contexts |
| Event | High | ✅✅✅✅ | Conferences, performances, broadcasts; CMOS 18/APA 8 requirements |

**Rationale for Event (v1.2 candidate):**
- **Factor 1 (Semantic):** Users distinguish "the conference" or "the concert" from a book or article. Differentiation from `SerialComponent` is based on the "live/ephemeral" nature vs. the "serialized/issued" nature of episodes in a series.
- **Factor 2 (Style):** CMOS 18 (14.165, 14.166) and APA 8 have dedicated rules for events/performances.
- **Factor 3 (Schema):** Requires `location` (venue), `date`, `network`, `organizer`, and a `series` relation for recurring events.
- **Factor 4 (No Parent):** The event is the primary entity; venue is a locator.


**Previously listed candidates now implemented:** Dataset (v1.1), Software (v1.1), Standard (v1.1). Preprint was added as `MonographType::Preprint` rather than a flat type, since the preprint server relationship is custodial rather than editorial.

## ContributorRole Extension Policy

New contributor roles should be added to the `ContributorRole` enum in `crates/citum-schema-data/src/reference/contributor.rs` when they represent a distinct agent role in a citation context. The enum includes both named variants for standardized roles and a `#[serde(untagged)] Custom(String)` escape hatch for domain-specific or unstandardized roles.

### Adding a Named Role

When adding a new named role variant to the `ContributorRole` enum:

1. **Justify the role.** Ensure it has a clear use case in at least one priority style (e.g., APA, Chicago, MLA, a domain-specific standard).

2. **Update the enum** in `crates/citum-schema-data/src/reference/contributor.rs`:
   ```rust
   pub enum ContributorRole {
       // existing roles ...
       YourNewRole,
       // ...
       #[serde(untagged)]
       Custom(String),
   }
   ```

3. **Update dispatcher logic.** If the new role affects the `author()` dispatch on `InputReference` (e.g., for work-level types like `AudioVisualWork`), update the pattern match in `crates/citum-schema-data/src/reference/mod.rs`.

4. **Update engine routing.** If the role requires distinct rendering or template variable mapping, add handling in `crates/citum-engine/src/values/contributor/mod.rs`.

### Using the Custom Variant

For unstandardized roles or domain-specific extensions, use the `Custom(String)` variant:

```yaml
contributors:
  - role: "uncommon-role"  # Parsed as Custom("uncommon-role")
    contributor:
      family: Smith
      given: Jane
```

This avoids schema churn for niche use cases. If a custom role becomes standardized across multiple styles, promote it to a named variant in a follow-up update.

## AudioVisualWork Rationale

`AudioVisualWork` was added as a first-class `InputReference` variant rather than a `MonographType` subtype because it has a distinct contributor model and requires type-aware primary author dispatch. Films and episodes have Directors as primary agents, recordings have Composers or Performers, and broadcasts have no fixed primary. This pattern differs fundamentally from monographs and should be followed for any future type where the primary agent role differs from the standard `author` field.

When considering new work-level types (e.g., `Artwork`, `MusicalWork`), use this principle: add a first-class variant and compose `WorkCore` if:
1. The primary contributor role is not `author`, OR
2. The type needs distinct rendering across major citation styles, OR
3. The field schema differs significantly from existing monolithic types

Otherwise, use a `MonographType` subtype for simplicity.

## References

- TYPE_SYSTEM_ARCHITECTURE.md - Full analysis of structural vs flat options
- biblatex manual (CTAN) - 31 entry types, flat model
- CSL 1.0 specification - 34 types, flat enumeration
- CLAUDE.md - Citum design principles

## Changelog

**v1.2 (2026-04-07):**
- Added ContributorRole Extension Policy section with guidance on adding named roles vs. using Custom variant
- Added AudioVisualWork Rationale section explaining work-level variant decisions

**v1.1 (2026-03-29):**
- Updated audit tables to reflect implemented types (Patent, Dataset, Standard, Software, MonographType subtypes)
- Updated future candidates (removed implemented types)
- Added Collection types to structural audit table
- Added cross-reference to ENUM_VOCABULARY_POLICY.md
- Fixed table alignment

**v1.0 (2026-02-14):**
- Initial policy based on legal citations architectural analysis
- 4-factor test established
- Decision flowchart added
- Example evaluations provided
