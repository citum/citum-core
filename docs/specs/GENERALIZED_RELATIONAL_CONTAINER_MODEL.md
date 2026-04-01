# Specification: Generalized Relational Container Model

**Status:** Active
**Date:** 2026-03-31
**Related:** [Chicago 18 / APA 8 Coverage Enhancements](./CHICAGO_18_COVERAGE.md)

## Purpose

This specification proposes a significant architectural shift in how Citum models bibliographic data. It advocates moving away from the "flat" variable structure inherited from CSL (e.g., `volume`, `volume-title`, `part-number`) toward a deeply relational, recursive model using `WorkRelation` for all bibliographic containers.

The goal is to give Citum a consistent recursive relation model for bibliographic containers while ensuring that the complexity of the model does not degrade the authoring experience for users writing YAML.

## Core Principle: The Document is the Part

In complex bibliographic hierarchies (Multivolume works, Serial installments), **the part is the document**. 

When citing "Part 1" of a serialized article or "Book 3" of a multivolume set, the physical document in hand is defined by that sequence identifier. 
- **Document Identity:** Its `title` is the part's title; its identifiers (`number`/`numbering`) define its specific identity.
- **Container Identity:** The Journal, Book Set, or Series is the `container`.

## Universal Relation Model: `WorkRelation`

While this specification focuses on the `container` hierarchy, **`WorkRelation` is the universal type for all bibliographic relations in Citum.** 

Any property that links one bibliographic entity to another (whether hierarchical or semantic) uses the `WorkRelation` enum. This ensures a consistent developer experience and uniform YAML/JSON behavior (supporting both inline embedding and citekey referencing) across the entire schema.

However, Citum distinguishes between the **functional behavior** of these relations based on their role in the schema:

1. **Hierarchical (`container`):** Models the primary parent-child relationship (e.g., Chapter in a Book). This relation is **structural**; it defines containment and triggers metadata inheritance (bubbling up).
2. **Associative (`original`, `reviewed`, `series`):** Models non-hierarchical, semantic links such as reprints, the subject of a review, or the recurring series of an event. These are maintained as **dedicated fields** rather than being collapsed into the `container` tree.
3. **Recursive & Polymorphic:** Because `WorkRelation` wraps the generic `InputReference`, any relation can point to any type of work.

## Relation Taxonomy: Hierarchical vs. Associative

To solve the duality between limited hierarchies (where a Chapter cannot contain a Book) and flexible relations (where any work can be reviewed), Citum implements a **Validation Layer** rather than rigid type refinement. The engine distinguishes between two functional categories to maintain logical integrity:

| Feature | Hierarchical (`container`) | Associative (`original`, `reviewed`, etc.) |
| :--- | :--- | :--- |
| **Logic** | Structural Containment (Tree/DAG) | Semantic Association (Graph) |
| **Inheritance** | **Yes:** Metadata bubbles up to parents | **No:** Independent metadata |
| **Type Constraints** | **Restricted:** Limited to "Container Types" | **Flexible:** Any valid `InputReference` |
| **Multiplicity** | **Singular:** Max one container per level | **Plural:** Can have multiple semantic links |

### The `WorkRelation` Struct

```rust
/// A relation to another bibliographic entity.
/// Untagged in serde to allow either an inline object or a string ID reference.
#[serde(untagged)]
pub enum WorkRelation {
    /// The target work is referenced by its ID (resolved at render time).
    Id(RefID),
    /// The target work is embedded inline.
    Embedded(Box<InputReference>),
}
```

## Structural Invariants (Validation Layer)

While the `WorkRelation` AST is polymorphic, the **Citum Validator** (`citum check`) and the engine enforce the following structural invariants to prevent logically impossible hierarchies:

1. **Acyclicity:** A work cannot be its own container, nor can any container be contained by its own descendant. (Associative relations like `original` also forbid self-reference but allow more complex graph topologies).
2. **Container Type Compatibility Matrix:** The `container` field is restricted by the following compatibility rules during ingest:

| Item Type | Valid `container` Targets |
| :--- | :--- |
| `CollectionComponent` | `Collection`, `Monograph`, `Archive` |
| `SerialComponent` | `Serial`, `Monograph` (for proceedings) |
| `Monograph` | `Monograph` (Set), `Collection`, `Series` |
| `LegalCase` | `Reporter`, `Serial` (Newspaper) |

3. **Inheritance Boundary:** Metadata inheritance ("bubbling up") **only** traverses the hierarchical `container` tree. It never follows associative links like `original` or `reviewed`.

## Metadata Inheritance (Bubbling Up)

A recursive container model must provide a robust metadata inheritance strategy. **Metadata inheritance is exclusive to the `container` relation.** If a document lacks a core property (e.g., `publisher`, `issued` date, `language`), the renderer should perform a **depth-first search** up through the `container` tree to find the value.

- **Primary Attributes:** Title, Author, and Numbering (Numbers) are typically specific to the item and **do not** inherit from parents.
- **Contextual Attributes:** Publisher, Publisher Place, Date, Language, and ISSN/ISBN **should** bubble up until a value is found or the root is reached.

## Formalizing "Numbering"

Analysis of CSL JSON fixtures reveals that `part-number` and `volume` are identity markers for the document. To handle multiplicity (e.g., "Supplement 2 of Volume 5"), we adopt a `Vec<Numbering>` model with a controlled vocabulary for reliable i18n rendering.

```rust
pub struct Numbering {
    /// Controlled vocabulary: "volume", "issue", "part", "supplement", "chapter", "book", "section"
    pub r#type: NumberingType, 
    pub value: String, // e.g., "4", "B", "12"
}
```

## Authoring Profiles (Syntactic Sugar)

To maintain YAML ergonomics, Citum separates the **Authoring Schema** from the **Engine AST**.

- **Syntactic Sugar:** The YAML/JSON loader accepts flat keys (e.g., `volume`, `issue`, `container-title`, `collection-title`) as a shorthand.
- **Canonical Storage:** During deserialization, shorthand numbering keys are normalized into canonical `numbering` entries. Serialized output uses `numbering`.
- **Authoring Profiles:** The loader uses predefined patterns to "up-sample" flat keys into the nested relational AST during ingest.
- **Embedding vs. Referencing:** Inline embedding is recommended for one-off hierarchies (e.g., book chapters). ID referencing is recommended for shared entities (e.g., a journal series or archival collection) to avoid duplication.

## Migration Requirements (`citum_migrate`)

The migration from flat CSL-style data to the recursive Citum model must be **type-aware** and predictable:
- **`article-journal`:** `volume` and `issue` map to specific intermediate `container` levels.
- **`chapter`:** `container-title` maps to a `book` container. `collection-title` maps to a nested `series` container.
- **`original-*`:** Flat prefix fields are migrated into a dedicated `original: WorkRelation` field.

## Acceptance Criteria

1. The legacy `Parent<T>` enum in `structural.rs` is removed.
2. `WorkRelation` is utilized for the recursive `container` property across all `InputReference` variants.
3. Metadata inheritance (bubbling up) logic is implemented for renderers, restricted to the `container` relation.
4. Structural validation (invariants and type compatibility) is implemented in `citum check`.
5. `Numbering` uses a controlled vocabulary and supports multiplicity (`Vec<Numbering>`).
6. The Citum loader supports "Syntactic Sugar" for flat YAML up-sampling.
7. `citum_migrate` provides type-aware relational migration for all major CSL item types.
