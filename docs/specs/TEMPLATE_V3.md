# Template Schema v3 Specification

**Status:** Draft
**Version:** 0.3
**Date:** 2026-05-05
**Supersedes:** `docs/specs/TEMPLATE_V2.md`
**Related:** csl26-t3v1, `docs/specs/DISTRIBUTED_RESOLVER.md`

## Purpose

**Audience:** Engine implementers and style authors.

The "hard-fork" nature of V2's `type-variants` (complete replacement with no inheritance) is incompatible with a distributed style ecosystem. V3 reintroduces **Structural Template Inheritance** using a pure diff-based model, so that every type-variant can be deterministically derived from the base template at resolution time.

This design explicitly **rejects "Macros"** to avoid the complexity and fragmentation of CSL 1.0. Instead, it relies on two pillars:
1.  **Surgical Diffs:** Type-variants that modify, add, or remove components from a base template.
2.  **Logic-Heavy Options:** Moving shared formatting logic (contributor lists, date fallbacks) into style-level configuration rather than template structures.

## Scope

**In Scope:**
- `extends` keyword within `type-variants` (defaulting to the base `template`).
- List-diff operations: `modify`, `add`, `remove`.
- Expansion of `options` (e.g., `contributor-config`, `date-config`) to absorb shared logic.
- Impact on `DistributedResolver` style-merging.

**Out of Scope:**
- Named templates or Macros (Forbidden).
- YAML Anchors (MAY be used locally for authoring convenience but MUST NOT be relied upon for cross-style reuse).

## Terminology

- **Template:** An ordered list of components.
- **Component:** A single rendering instruction (e.g., `contributor: author`, `date: issued`, alongside rendering hints like `prefix` or `form`).
- **Type-variant:** A named diff that transforms the base template into a specialized template for a specific reference type (e.g., `article-journal`).

---

## Design

### §1 — The Structural Diff Model

In V3, every `type-variant` is a transformation of a parent template. By recording the **intent of the change** rather than a copy of the result, we ensure that updates to the parent style flow through to the variant.

```yaml
bibliography:
  template:
    - contributor: author
    - date: issued
      form: year
    - title: primary
    - variable: publisher
    - variable: url

  type-variants:
    article-journal:
      # If `extends` is omitted, the variant implicitly extends the base `template`.
      modify:
        - match: { variable: publisher }
          suppress: true
      add:
        - after: { title: primary }
          component: { title: parent-serial, emph: true }
```

If `extends` is omitted, the variant implicitly extends the base `template`. Optionally, `extends` MAY reference another type-variant of the same template, in which case the parent variant's diffs are applied before the child's.

### §2 — Absorbing Macros into Style Options

The primary reason authors use macros in other systems is to ensure consistent formatting for complex entities (like a list of 10 authors). Citum solves this by moving that logic into `options`.

#### 2.1 Contributor Configuration
Instead of a "Macro" for author formatting, authors configure the `contributor-config` once. Rendering of any component with `contributor: <role>` MUST be governed by `options.contributors.<role>` unless that component explicitly overrides one of these policies with a local hint (e.g., a local `delimiter`).

```yaml
options:
  contributors:
    author:
      shorten: { min: 3, use-first: 1 }
      and: "symbol"
      delimiter: ", "
      et-al-use-last: true
```

#### 2.2 Date Configuration
Templates SHOULD reference logical date roles (e.g., `date: issued`) while the engine resolves concrete fallbacks using `options.dates`. This keeps fallback logic centralized and prevents macro-like duplication in templates.

```yaml
options:
  dates:
    primary:
      order: [issued, event-date, available]
      missing: "omit"
```

### §3 — Merge Operations (Formalized)

The engine MUST process each operation list (`modify`, `remove`, `add`) in the order provided. The order of these keys (`modify`, `remove`, `add`) within a variant has no semantic effect.

1.  **Identify Anchor (Match):** A `match` selector is a partial match: a component matches if it contains all key-value pairs specified in `match`, with equal values, regardless of any additional keys on the component.
    *   If no component matches, the operation MUST be ignored or treated as a validation error (implementation-defined, but validators SHOULD treat this as an error).
    *   If multiple components match, engines MUST treat this as a validation error or select the first match deterministically; style authors SHOULD avoid ambiguous `match` selectors.
2.  **Apply Operation:**
    *   `modify`: Overwrites rendering hints. If a `modify` operation attempts to change the component’s kind or primary value (e.g., `contributor: author` to `contributor: editor` or `variable: publisher` to `variable: url`), the style is invalid and must be rejected by validators or ignored by non-validating engines (implementation-defined).
    *   `remove`: Deletes the anchor from the list.
    *   `add`: Inserts a new component `before` or `after` the anchor. An `add` operation MAY specify either `before` or `after`, but not both. If both are present, the style is invalid. If the anchor in `before`/`after` does not match any component, the engine MUST append the new component to the end of the list.

### §4 — Distributed Merging

Resolution (`try_into_resolved_with`) follows the URI chain. Resolution is recursive: if the parent style itself `extends` another style, the engine MUST fully resolve that ancestor chain before applying the child’s diffs.

When a child style `extends` a remote parent:
1.  Fetches and fully resolves the remote parent's templates.
2.  Applies the parent's `type-variants` (if any).
3.  Applies the child's `type-variants` diffs to the fully resolved parent template.

**Example: Subscriber Style (`university-apa.yaml`)**
```yaml
extends: https://hub.citum.org/styles/apa.yaml

bibliography:
  # No local 'template:' is defined; it is inherited from the parent.
  type-variants:
    article-journal:
      # Inherits the article-journal from APA, then adds a localized label:
      add:
        - before: { variable: doi }
          component: { term: doi, suffix: ": " }
```

Engines SHOULD treat unreachable or invalid parent URIs as resolution errors; style authors MUST NOT assume offline resolution if remote parents are unavailable.

---

## Acceptance Criteria

- [x] Macros are absent from the spec.
- [ ] `type-variants` schema supports `extends`, `modify`, `add`, and `remove` with defined matching and ordering semantics.
- [ ] Style-level `options` expanded to handle contributor and date formatting policies, with clear precedence rules vs local component hints.
- [ ] Engine resolution logic supports cross-URI template diffing, including recursive parent chains and error handling for missing parents.

---

## Changelog

- v0.3 (2026-05-05): Clarified terminology, matching semantics, order of operations, and validation rules. Added subscriber style example using localized terms instead of literal affixes.
- v0.2 (2026-05-05): Pivoted to Pure Diff model. Removed Macros/Named Templates. Expanded role of style-level options.
- v0.1 (2026-05-05): Initial draft (Macro-based).
