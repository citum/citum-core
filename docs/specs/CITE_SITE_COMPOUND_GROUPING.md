# Spec: Cite-Site Compound Grouping Overrides

**Status:** Active
**Date:** 2026-04-14
**Bean:** csl26-di0r

## 1. Motivation

Citum currently supports "static" compound sets (pre-defined in the bibliography or via the `Processor` API). However, certain scientific domains (specifically Chemistry and Physics) require "dynamic" or "cite-site" grouping.

This pattern is often referred to as "numeric compound styles" in CSL terminology. While the CSL standard itself does not currently support cite-site grouping, Citum implements this as a first-class extension to its numeric compound style support. In these workflows, a user decides at the moment of citation that several distinct references should be clustered under a single bibliography number, often with sub-labels (e.g., `2a-c`). This is traditionally handled in LaTeX by the `mciteplus` package.

**Note:** This spec is explicitly focused on dynamic grouping at cite-time and assumes a `numeric` or `compound-numeric` style. Author-date or label styles generally do not support this grouping behavior.

## 2. Proposed Changes

Instead of adding per-item flags (e.g., `is_tail`), we propose marking the `Citation` as a whole as a grouped or compound cluster. The order of the `CitationItem`s inherently determines the "head" and "tail" roles: the first item is the head, and all subsequent items are merged as tails.

### 2.1. Schema Changes (`citum-schema`)

Add a grouping flag to the `Citation` object (which represents a cluster of references cited at a single location). This flag is intended to be stable across renders and persisted in JSON/YAML interchange as `"grouped": true`.

```rust
pub struct Citation {
    pub citation_id: String,
    pub citation_items: Vec<CitationItem>,
    pub properties: CitationProperties,
    
    /// If true, the entire citation is treated as a single dynamic compound set.
    /// The first item in `citation_items` acts as the head, and subsequent
    /// items are merged into its bibliography entry as tails.
    #[serde(default, skip_serializing_if = "is_false")]
    pub grouped: bool,
}
```

**Invariants:**
- In a `grouped` citation, implementations MUST NOT reorder `citation_items`, even if the style ordinarily sorts citations.
- The `grouped` flag is a hint for *this citation cluster*. Whether and how a style actually renders a compound bibliography entry is strictly controlled in the style layer (e.g., via `compound-numeric` bibliography options).

### 2.2. Processor Behavior (`citum-engine`)

When the `Processor` encounters a `Citation` with `grouped: true`:

1. **Group Identification:** It treats `citation_items[0]` as the "head" reference.
2. **Dynamic Set Creation:** It internally merges `citation_items[1..]` into a temporary compound set attached to the "head". This set is scoped strictly to that render pass and does *not* modify any underlying static `CompoundSet` definitions.
3. **Bibliography Rendering:** The "tail" items are rendered within the same bibliography entry as the "head". The exact formatting (e.g., `(1a, 1b)` versus `(1a, b)`) depends on the style's `compound-numeric` delimiters and sub-label settings (see `docs/architecture/CSL26_ZAFV_NUMERIC_COMPOUND_CITATIONS.md`).
4. **Numbering:** All items in the dynamic group share the same primary bibliography number.

### 2.3. Conflict Resolution

Static definitions always take precedence over dynamic cite-site grouping requests. 

Implementations *should* emit a logged warning (or surfaced message in debug mode) when a cite-site group conflicts with a static compound set, and may ignore the dynamic grouping entirely in that case.

A "contradiction" occurs if:
- A dynamic group has the same head as a static set, but different tails.
- A reference is a tail in a static set but acts as a head in a dynamic group.
- Items belonging to two different static sets are grouped together dynamically.

## 3. Input Syntax (Integrations)

Integrators (like a Pandoc filter or Djot processor) need a way to express grouped clusters. The normative representation is the AST flag on the citation node; markdown syntax is just one encoding.

- **Markup Options:**
  - Introduce a cluster-level modifier in markdown (e.g., `[*@ref1; @ref2; @ref3]`). The `*` is purely an encoding signal and never appears in the rendered output.
- **Translation:**
  - The integration parses the cluster, creates a single `Citation`, and sets `grouped: true`.

## 4. Advantages of Citation-Level Grouping

- **Simplicity:** Conceptually cleaner at the schema level. Eliminates invalid states like `is_tail: true` on the first item in a citation.
- **Order-Based:** Naturally leverages the existing array order of `citationItems` to establish head vs. tail relationships.
- **Atomic:** Treats the compound reference as a single logical unit at the point of citation.
- **Style-Friendly:** Grouping is orthogonal to citation style, so styles can support, ignore, or partially support compound-numeric behavior without changing the schema.

## 5. Design Decisions (Unresolved Questions Addressed)

To ensure consistent implementation behavior, the following defaults are recommended:

### 5.1. Mixed Citations
Citum will not support per-item grouping flags in a single cluster. Authors and integrators must split mixed references (e.g., `[1, 2a-c, 3]`) into separate grouped and ungrouped citation nodes.
- **Source:** `[@ref1] [*@ref2; @ref3; @ref4] [@ref5]`
- **AST:** Three distinct `Citation` objects: 
  1. `[ref1]` (`grouped: false`)
  2. `[ref2, ref3, ref4]` (`grouped: true`)
  3. `[ref5]` (`grouped: false`)

### 5.2. Cross-Citation Consistency
- The **first occurrence** of a reference establishes its grouping. If `grouped: true` is used first, it establishes the bibliography grouping for that head; later citations to any member share that entry/number.
- If the *first* occurrence is ungrouped and a later citation attempts to group it, the engine prefers the first occurrence (ignoring the later grouping request) and issues a warning.
- **Single-member follow-ups:** Citing a member alone (e.g., `ref2`) after it has been grouped with `ref1` will display as its specific sub-item (e.g., `[2b]`).

---

## 6. Changelog

- **2026-04-14**: Initial draft spec for citation-level grouping overrides.
