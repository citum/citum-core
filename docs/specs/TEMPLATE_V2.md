# Template Schema v2 Specification

**Status:** Active
**Version:** 1.5
**Date:** 2026-03-25
**Supersedes:** (none)
**Related:** csl26-da9f, csl26-ww2m, csl26-96wg, csl26-boe8, csl26-8eom, csl26-rrag

## Purpose

The primary goal of v2 is to **simplify the template model**. Templates should
be lightweight composition: a flat or grouped list of component references with
rendering hints (`prefix`, `suffix`, `emph`, `quote`, `wrap`, `form`). Complex
per-component configuration — name shortening, role labels, date fallback
chains, link policy — belongs at the style level, not inside individual template
components.

A compact string DSL is the proof-of-concept that this simplification worked:
if any template can be expressed in a single line, the model is correct. The
DSL is a design litmus test and a follow-on authoring affordance, not a primary
deliverable.

This spec records the v2 design direction and the current state of `main`.
On `main` today, `items` has been renamed to `group`, `type-variants` exists
on citation and bibliography specs, `Style::validate()` exists, and the old
template-level `ComponentOverride` / per-component `overrides` mechanism is
already gone.

This spec uses **template overrides** to mean the removed per-component
template mechanism only. It does **not** refer to other override surfaces such
as `options.substitute.overrides`, which are still live and represent a
different concept.

## Scope

**Landed on `main`:**
- Rename the `items` YAML key to `group` (struct rename + serde alias).
- Add `type-variants` to `CitationSpec`.
- Add `type_variants` to `BibliographySpec`.
- Add `TypeSelector` validation helpers and `Style::validate()` returning `Vec<SchemaWarning>`.
- Catalog attributes that move from template components to style-level config (§3).
- Compact string DSL design as a simplification litmus test (§4).
- Pain point audit with effort estimates and ownership decisions (§5).

**Not landed on `main`:**
- Compact DSL parser implementation.
- Follow-on design work from the pain-point audit, such as split option schemas
  and a wrap-owned inner-affix model.

**Reality snapshot (2026-03-25):**
- `ComponentOverride` and per-component template `overrides` are absent from
  `crates/`.
- 15 styles in `styles/` still contain `overrides:`, but those are
  `options.substitute.overrides`, not template-component overrides.
- `Style::validate()` and `SchemaWarning` are implemented.
- This spec no longer claims `BibliographySpec` parses `type-templates` via serde alias,
  because that alias is not present on `main`.

**Out of scope:**
- Changing `TemplateComponent` discrimination (untagged serde remains).
- Altering CSL group-suppression semantics.
- Defining exact structure of `contributor-config`, `date-config`, `link-policy`
  (follow-on specs).
- Implementing the compact DSL parser (follow-on task, once §3 config structures
  are specced).
- Resolving the duplicate `Volume`/`Number` variable split.

---

## Design

### §1 — `items` → `group` Rename

#### 1.1 Problem

The YAML key `- items: [...]` on a `TemplateList` is borrowed from generic
data modeling. The concept it represents — a grouped sequence of components
that suppresses entirely if none of its children produce output — is a
first-class citation concept called a **group** (CSL 1.0 `<group>`). The
current name undersells this semantics and makes styles harder to read.

#### 1.2 Decision

Pure rename. No semantic change.

| Layer | Before | After |
|-------|--------|-------|
| YAML key | `- items: [...]` | `- group: [...]` |
| Rust struct | `TemplateList` | `TemplateGroup` |
| Rust field | `pub items:` | `pub group:` |
| Serde compat | — | `#[serde(alias = "items")]` on the field |

The serde alias keeps all existing YAML parseable without a migration wave.
New styles and migration output use `group`. The alias is permanent — not
removed until a future major schema version.

#### 1.3 CSL Group Semantics (normative)

A `group` suppresses its entire output — including prefix, suffix, wrap, and
delimiter — if **none** of its child components produce any text. A child
produces text if it has a non-empty rendered value after all suppression rules
are applied. This matches CSL 1.0 `<group>` behavior.

```yaml
# This group renders nothing if both title: parent-serial
# and the volume/issue sub-group are absent.
- group:
    - title: parent-serial
      emph: true
    - group:
        - number: volume
          emph: true
        - number: issue
          wrap: parentheses
      delimiter: ""
  delimiter: ", "
```

#### 1.4 Migration Path

- All existing `.yaml` styles parse without change (alias covers them).
- `citum-migrate` output switches to `group:` key immediately after the
  Rust rename lands.
- A one-time bulk rename pass on existing styles in `styles/` can be done
  with `sed -i 's/^  - items:/  - group:/g'` but is not required for
  correctness and may be deferred to a later cleanup wave.

#### 1.5 GUI Implications

GUI builders should present a `group` node as a collapsible panel labeled
"Group" with a child list editor. The group's own rendering fields (prefix,
suffix, wrap, delimiter) appear as panel-level controls distinct from the
child items. The suppression-on-empty behavior should be surfaced as a
non-editable tooltip or badge: "Hides automatically if all children are
empty."

#### 1.6 Rust Change Summary

```rust
// crates/citum-schema-style/src/template.rs

// Before
pub struct TemplateList {
    pub items: Vec<TemplateComponent>,
    ...
}

// After
pub struct TemplateGroup {
    #[serde(alias = "items")]
    pub group: Vec<TemplateComponent>,
    ...
}

// TemplateComponent enum
// Before: List(TemplateList)
// After:  Group(TemplateGroup)
```

All match arms on `TemplateComponent::List(...)` become
`TemplateComponent::Group(...)`. The `dispatch_component!` macro and any
engine code matching on `List` must be updated in the same commit.

---

### §2 — Remove `overrides`; Promote `type-variants`

**Design target:** Option A (full removal) remains the intended end state.

**Current `main` reality:** `type-variants` is available and the old
per-component `overrides` / `ComponentOverride` mechanism is already removed
from the template schema.

#### 2.1 Current Mechanism

Every `TemplateComponent` variant carries:

```rust
pub overrides: Option<HashMap<TypeSelector, ComponentOverride>>
```

where `ComponentOverride` is:

```rust
pub enum ComponentOverride {
    Component(Box<TemplateComponent>),  // full component replacement
    Rendering(Rendering),               // rendering-only patch
}
```

This makes type-specific behavior local to each component, scattered across the
template tree:

```yaml
# Today: type-specific behavior buried inside each component
- contributor: author
  form: long
  overrides:
    legal-case: { suppress: true }
    personal-communication:
      contributor: author
      form: long
      name-order: given-first
      suffix: ", personal communication"
- date: issued
  form: year
  overrides:
    personal-communication:
      date: issued
      form: year-month-day
```

#### 2.2 Why Removed

1. **Scanning cost.** A reader must visit every component to understand what a
   given type renders. A GUI must re-parse the full template tree per type.
2. **No top-level view.** There is no single place that shows "here is what a
   `personal-communication` renders differently."
3. **Semantic ambiguity.** `ComponentOverride::Component(Box<TemplateComponent>)`
   allows replacing a `contributor` with a `date`. Semantically wrong, parses silently.
4. **GUI hostility.** Any GUI wanting a per-type preview must traverse the entire
   component tree, merge overrides, and reconstruct a virtual template. No clean
   data model to bind to a tab panel.
5. **DSL incompatibility.** A component with `overrides` cannot be expressed in
   the compact DSL at all, even if the component itself is simple — the override
   is a sub-object of arbitrary depth.
6. **Simplification blocker.** Per-component `overrides` are the single largest
   source of template model complexity. Removing them is a prerequisite for §3.

#### 2.3 Replacement: `type-variants` at the Spec Level

The replacement model is still `type-variants` at the spec level:

| Spec | Before | After |
|------|--------|-------|
| `BibliographySpec` | `type_templates` in older drafts | `type_variants` field on `main` |
| `CitationSpec` | (absent) | `type_variants` field on `main` |

`type-variants` maps a `TypeSelector` key to `Vec<TemplateComponent>` (the
`Template` type alias) — the same flat list of components used in the default
`template` field. It is **not** a nested `CitationSpec` or `BibliographySpec`.
An empty value (`[]`) means suppress entirely for that type.

```rust
// BibliographySpec — the value type is Vec<TemplateComponent>, same as Template
pub type_variants: Option<HashMap<TypeSelector, Template>>,

// CitationSpec — new field, same type
pub type_variants: Option<HashMap<TypeSelector, Template>>,
```

Runtime: if the reference type matches a `type-variants` key, use that template;
otherwise fall through to the default `template`. The matched template is used
in full — there is no field merging between a `type-variants` entry and the
default template.

**Mode resolution for citations:** type-variant lookup happens after
`integral`/`non-integral` mode resolution. If both `citation.type-variants`
and `citation.integral.type-variants` have an entry for the same type, the
mode-specific one wins entirely — no merging.

#### 2.4 Migration Gap: "Suppress for N Types" Pattern

The most common use of `overrides` in existing styles is:

```yaml
- contributor: author
  overrides:
    legal-case: { suppress: true }
```

Under the landed removal model, this requires duplicating the full default template into a
`type-variants` entry that simply omits the suppressed component:

```yaml
type-variants:
  legal-case:
    - date: issued        # all components except contributor: author
      form: year
    - title: primary
      ...
```

This duplication is the main migration cost. `compile_bibliography_with_types`
in `citum-migrate` must be updated to generate this pattern automatically — it
must not require hand-duplication when the only change is suppressing one
component for a type. The migration compiler fix and the schema field removal
must land in the same commit so no intermediate state is pushed where old-style
`overrides` YAML would need to parse against a schema that no longer accepts it.

#### 2.5 Before/After: APA Personal Communication

**Before:**
```yaml
citation:
  non-integral:
    template:
      - contributor: author
        form: short
        overrides:
          personal-communication:
            contributor: author
            form: long
            name-order: given-first
            suffix: ", personal communication"
      - date: issued
        form: year
        overrides:
          personal-communication:
            date: issued
            form: year-month-day
```

**After:**
```yaml
citation:
  type-variants:
    personal-communication:
      - contributor: author
        form: long
        name-order: given-first
        suffix: ", personal communication"
      - date: issued
        form: year-month-day
  non-integral:
    template:
      - contributor: author
        form: short
      - date: issued
        form: year
      - variable: locator
```

#### 2.6 Before/After: APA Legal Case (Suppress Pattern)

**Before:**
```yaml
bibliography:
  template:
    - contributor: author
      form: long
      overrides:
        legal-case: { suppress: true }
    - date: issued
      form: year
      overrides:
        legal-case: { suppress: true }
    - title: primary
      emph: true
      overrides:
        legal-case:
          title: primary
          emph: false
          quote: false
```

**After** (full alternate template; compiler must generate this from the suppress pattern):
```yaml
bibliography:
  type-variants:
    legal-case:
      - title: primary
        emph: false
        quote: false
        suffix: ", "
      - number: volume
        suffix: " "
      - variable: reporter
        suffix: " "
      - number: pages
      - group:
          - variable: authority
          - date: issued
            form: year
        delimiter: " "
        wrap: parentheses
        prefix: " "
        suffix: "."
  template:
    - contributor: author
      form: long
    - date: issued
      form: year
    ...
```

#### 2.7 GUI Implications

`type-variants` maps naturally to a tab panel:
- Default tab: main `template`.
- Each `type-variants` key: additional tab labeled with the type name.
- Empty `[]`: "Suppress for this type" toggle.

No secondary component-level override UI is needed.

#### 2.8 `citum-migrate` Impact

- `compile_bibliography_with_types`: field rename `type_templates` →
  `type_variants` in output; update suppress-pattern generation (§2.4).
- `compile_for_type` in `compilation.rs`: stop emitting `overrides`; all
  type-branching becomes `type-variants` including at the citation level.
- The `ComponentOverride` enum and the `overrides` field on every
  `TemplateComponent` variant are removed from `citum-schema-style`.

---

### §3 — Template Simplification Catalog

The simplest template component is: `kind` + `value` + rendering hints. That is
all a template should need to express. This section catalogs which attributes
currently live on template components but belong at the style level, and which
must stay because they are inherently positional.

#### 3.1 What a Simplified Template Component Looks Like

```yaml
# Simplified: kind, value, and rendering hints only
- contributor: author
  form: short
- date: issued
  form: year
  wrap: parentheses
- variable: locator
  prefix: ", "
```

No name-shortening config, no role labels, no fallback chains — just what to
show and how to punctuate it at this position.

#### 3.2 Attributes That Move to Style-Level Config

| Attribute | Currently on | Proposed home | Notes |
|-----------|-------------|---------------|-------|
| `shorten` | `TemplateContributor` | `contributor-config` | Name truncation is a style-wide policy |
| `label` (role label) | `TemplateContributor` | `contributor-config` | e.g., "Ed." suffix — style-wide |
| `and`, `delimiter`, `sort-separator` | `TemplateContributor` | `contributor-config` | List formatting — style-wide |
| `fallback` | `TemplateDate` | `date-config` or variable chain | Date substitution chain is style-wide |
| `links` | any component | `link-policy` | Link wrapping rules — style-wide |

These attributes are the same for every occurrence of the same contributor role
or date variable across a style. There is no meaningful case where `author`
shortening in the citation differs from `author` shortening in the bibliography.
Making them style-level eliminates redundancy and reduces template verbosity.

#### 3.3 Attributes That Stay in Templates

The following are inherently **positional** — their value depends on where in
the output the component appears:

| Attribute | Reason to stay |
|-----------|---------------|
| `form` | The same variable may appear in long form (bibliography) and short form (citation) |
| `name-order` | May differ by template position (e.g., inverted only in bibliography) |
| `emph`, `quote`, `wrap` | Rendering punctuation is position-specific |
| `prefix`, `suffix` | Delimiter punctuation is position-specific |
| `disambiguate-only` | Affects what renders at *this position* when disambiguation fires |

#### 3.4 Scope of This Spec

This spec establishes the **intent and direction** for template simplification.
The exact structure of `contributor-config`, `date-config`, and `link-policy`
are out of scope here — each requires its own follow-on spec. The purpose of
§3 is to unblock the DSL design (§4): once these attributes move out, the DSL's
limitations list shrinks to near-zero.

Existing styles that use `shorten`, `label`, `fallback`, and `links` on template
components continue to parse via serde compatibility aliases after v2 lands.
Migration to style-level config is a follow-on wave.

---

### §4 — Compact String Template DSL (Litmus Test)

If the simplification in §3 lands, any template component reduces to
`kind:value` + optional modifiers + optional rendering hints. A compact string
can represent this without loss. This section designs the DSL as a
**proof-of-concept**: if any template cannot be expressed in compact form, that
reveals residual complexity that should be moved out of the template.

The DSL parser is a follow-on implementation task. This section defines the
syntax so the design can be validated against real styles now.

#### 4.1 Syntax

Compact templates are a sequence of **component tokens** separated by ` ; `
(semicolon with spaces — see §4.1.1).

```
<template>       ::= <component-token> [" ; " <component-token>]*
<component-token> ::= <kind>:<value>[/<modifier>]* [+<hint>]*
<kind>           ::= contributor | date | title | number | variable | term | group
<modifier>       ::= secondary field value (form, wrap, name-order, etc.)
<hint>           ::= prefix:<str> | suffix:<str> | emph | quote | suppress | wrap:<str>
```

- **Colon** (`:`) separates kind from value within a token.
- **Slash** (`/`) introduces modifiers (secondary fields).
- **Plus** (`+`) introduces rendering hints.
- **Groups** use `group(...)` with comma-separated interior — function-call
  syntax is intentional: a group is a higher-order container, not a field
  reference, and the parens make the nesting boundary unambiguous.

| Full YAML | Compact |
|-----------|---------|
| `contributor: author` | `contributor:author` |
| `contributor: author` + `form: short` | `contributor:author/short` |
| `date: issued` + `form: year` | `date:issued/year` |
| `date: issued` + `form: year` + `wrap: parentheses` | `date:issued/year+wrap:parentheses` |
| `title: primary` + `emph: true` | `title:primary+emph` |
| `number: volume` | `number:volume` |
| `variable: locator` | `variable:locator` |
| `variable: publisher` + `prefix: ". "` | `variable:publisher+prefix:". "` |

**Group example:**
```
group(date:issued/year, variable:locator)+wrap:parentheses+delimiter:", "
```

#### 4.1.1 Separator: Semicolon (confirmed)

` ; ` (semicolon with spaces) is the confirmed component separator.

- Semicolons never appear in single-reference citation output at the
  component-separator level (they appear in multi-cite output only).
- No escape needed in practice: a semicolon inside a prefix/suffix string
  (e.g., `prefix:"; "`) is protected by the surrounding quotes.
- The parser must tokenize quoted string literals **before** splitting on `;`.
  `prefix:"; "` must not split at the `;` inside the quoted string.

#### 4.2 Concrete Examples

**Example 1 — APA non-integral citation:**
```
contributor:author/short ; date:issued/year+wrap:parentheses ; variable:locator
```

Full YAML equivalent:
```yaml
- contributor: author
  form: short
- date: issued
  form: year
  wrap: parentheses
- variable: locator
```

**Example 2 — Chicago author-date citation:**
```
contributor:author/short ; date:issued/year ; variable:locator+prefix:", "
```

**Example 3 — Retrieved-from group:**
```
group(date:accessed/year-month-day+prefix:r ; term:retrieved-from ; variable:url)+delimiter:" "+prefix:" "
```

Full YAML equivalent:
```yaml
- group:
    - date: accessed
      form: year-month-day
    - term: retrieved-from
    - variable: url
  delimiter: " "
  prefix: " "
```

#### 4.3 Parser Location and Error Handling

The parser will live at `crates/citum-schema-style/src/template_dsl.rs`.

Public API (design):
```rust
pub fn parse_compact_template(s: &str) -> Result<Vec<TemplateComponent>, TemplateDslError>
```

Error variants:
- `UnknownKind` — kind not in the allowed set.
- `UnknownModifier { kind, modifier }` — modifier not valid for that kind.
- `UnclosedGroup` — `group(` without matching `)`.
- `UnterminatedString` — quoted string opened but not closed (catches
  `prefix:"; ` with missing closing quote before the parser reaches `;`).
- `GroupDepthExceeded` — nesting beyond limit (e.g., 5 levels).

The parser is a separate API surface for GUI tools; it is not invoked during
normal YAML deserialization.

#### 4.4 Residual Cases Requiring Full YAML

After §3 simplification, the only expected residual is:

- **`disambiguate-only`** — inherently positional (affects what renders at this
  position during disambiguation); cannot be a style-level setting.

If any other attribute is found that cannot be expressed in compact form, it
should be evaluated for migration to style-level config per §3 before adding
DSL syntax for it.

---

### §5 — Additional Pain Points (Audit)

#### P1 — Stringly-Typed `TypeSelector` (Implemented on `main`)

**Problem.** `TypeSelector` accepts any string. Typos (e.g., `article_journal`)
silently match nothing.

**Current state.** `validate_type_name(s: &str)` exists and `Style::validate()`
emits `SchemaWarning::UnknownTypeName` for unknown names discovered in
`type_variants`.

**Effort:** Low.

---

#### P2 — Two Concepts Named "Type Override" (Implemented; moot on `main`)

**Problem.** Per-component `overrides` and spec-level `type-variants` both
expressed "type-specific behavior" with no naming distinction.

**Resolution.** Moot on `main`: template-level `overrides` is gone, so
spec-level `type-variants` is now the only template branching surface.
This does not affect other override concepts such as
`options.substitute.overrides`.

---

#### P3 — `CitationSpec.options` Accepts Bibliography-Only Fields (Deferred follow-on: `csl26-8eom`)

**Problem.** `Option<Config>` on both specs includes bibliography-only fields
that are silently ignored in a citation context.

**Current state.** Still live on `main`.

**Planned resolution.** Introduce `CitationOptions` / `BibliographyOptions`.
Deferred; tracked by `csl26-8eom`.

---

#### P4 — Duplicate Variables Across `SimpleVariable` and `NumberVariable` (Partially addressed in docs)

**Problem.** `Volume`, `Number`, etc. appear in both enums; authors cannot
predict which to use.

**Current state.** Still live in the type model, but this PR documents the
authoring rule in schema comments and the style author guide:
use `number:` when numeric formatting or numeric labels matter, and `variable:`
for string passthrough fields.

**Follow-on.** Keep the underlying model as-is for now; no separate bean is
needed unless the duplication itself becomes an implementation target.

---

#### P5 — `inner-prefix`/`inner-suffix` Not Tied to `wrap` (Deferred follow-on: `csl26-rrag`)

**Problem.** Fields silently ignored without `wrap`. GUI cannot enforce the
dependency.

**Current state.** Still live on `main`.

**Planned resolution.** Fold inner affixes into a wrap-owned structure so the
dependency is explicit in the data model. Deferred; tracked by `csl26-rrag`.

---

#### P6 — `ComponentOverride::Component` Cross-Kind Replacement (Implemented; moot on `main`)

**Problem.** Replacing a `contributor` with a `date` override parses silently.

**Resolution.** Moot on `main`: `ComponentOverride` no longer exists in the
template schema.

---

#### P7 — No `Style::validate()` Method (Partially implemented on `main`)

**Problem.** All validation is at parse time or silently at render time.

**Current state.** `Style::validate(&self) -> Vec<SchemaWarning>` exists, but
today it only covers a narrow first slice of schema validation, led by P1's
unknown type-name warnings.

**Effort:** Medium.

---

## Implementation Notes

### Status Table

| Status | Work |
|--------|------|
| Landed on `main` | `items` → `group` rename |
| Landed on `main` | `type-variants` added to `CitationSpec` |
| Landed on `main` | `type_variants` field present on `BibliographySpec` |
| Landed on `main` | `TypeSelector` validation helpers + `Style::validate()` initial implementation |
| Deferred follow-on | Split `CitationOptions` / `BibliographyOptions` (`csl26-8eom`) |
| Deferred follow-on | Tie inner affixes to wrap model (`csl26-rrag`) |
| Follow-on design work | Compact DSL parser after §3 config structures are fully specced |

### Schema Regeneration

After any `template.rs` or `lib.rs` change:

```bash
cargo run --bin citum --features schema -- schema --out-dir docs/schemas
git add docs/schemas/
```

Include a `Schema-Bump: patch` footer (new optional fields → patch).
Step 5 warrants `Schema-Bump: major` (field removal).

### Key File Locations

| Change | File |
|--------|------|
| `TemplateGroup` rename | `crates/citum-schema-style/src/template.rs` |
| `CitationSpec.type_variants`, `Style::validate` | `crates/citum-schema-style/src/lib.rs` |
| Compact DSL parser (follow-on) | `crates/citum-schema-style/src/template_dsl.rs` (new) |
| Migration output field + suppress pattern | `crates/citum-migrate/src/template_compiler/bibliography.rs` |
| Citation-level type-variants emission | `crates/citum-migrate/src/template_compiler/compilation.rs` |

---

## Acceptance Criteria

- [x] `- group:` and `- items:` both parse to `TemplateComponent::Group`.
- [x] `TemplateGroup` serializes as `group:`, never `items:`.
- [x] `BibliographySpec` exposes `type_variants` on `main`.
- [x] `CitationSpec.type_variants` accepts a `HashMap<TypeSelector, Template>`
      where `Template = Vec<TemplateComponent>`.
- [x] Per-component `overrides` field is absent from the template schema.
- [x] `ComponentOverride` is absent from the template schema.
- [x] `style/apa-7th.yaml` renders all 12 oracle scenarios without regression.
- [x] `style/chicago-author-date.yaml` renders all 12 oracle scenarios without
      regression.
- [x] `Style::validate()` emits a `SchemaWarning` for unrecognized type names
      in `TypeSelector`.
- [x] Schema and author docs explain the current `number:` vs `variable:` split.
- [ ] (DSL — follow-on) `parse_compact_template("contributor:author/short ; date:issued/year+wrap:parentheses")`
      returns a `Vec<TemplateComponent>` with two elements matching the
      expected YAML AST.
- [ ] (DSL — follow-on) `parse_compact_template("group(date:issued/year, variable:url)+prefix:\" \"")`
      returns a `TemplateGroup` with two children.
- [ ] (DSL — follow-on) `parse_compact_template("unknown:foo")` returns
      `Err(TemplateDslError::UnknownKind)`.
- [x] Schema JSON regenerated for current doc-comment reality.
- [ ] **(Future implementation PR)** Breaking removal tagged `Schema-Bump: major`.
- [ ] **(Future implementation PR)** All existing nextest suite passes without regression.
- [ ] No new style authored after v2 uses `shorten`, `label`, `fallback`, or
      `links` on a template component (lint or doc convention).

---

## Open Questions

All three design questions are resolved, even though not all dependent
implementation work has landed:

1. **`overrides` disposition:** Option A (full removal) — confirmed.
2. **DSL separator:** Semicolon (` ; `) — confirmed.
3. **`type-variants` mode resolution:** Mode-specific sub-spec wins entirely
   (no merging) — confirmed.

---

## Changelog

- v1.5 (2026-03-25): Reconciled the spec to `main` reality. Removed incorrect
  claims that `type-templates` aliasing exists on `main`, linked P3 and P5 to new beans
  (`csl26-8eom`, `csl26-rrag`), and documented the current `number:` vs
  `variable:` authoring rule. Corrected the spec after verifying that
  `ComponentOverride` and template-component `overrides` are already absent
  from `crates/`. Added `csl26-boe8` as the reconciliation bean.
- v1.3 (2026-03-23): Marked Active. Steps 1–4 landed in PR `spec/template-v2`
  (Schema-Bump: patch). Step 5 (overrides removal) deferred — noted in Scope,
  §2, Ordering, and Acceptance Criteria. Acceptance criteria checked off for
  completed items.
- v1.2 (2026-03-23): Reframed as simplification-first. Added §3 (Template
  Simplification Catalog). DSL (now §4) reframed as litmus test; separator
  confirmed as semicolon; tokenization note and `UnterminatedString` error
  added. Option A confirmed; §2.4 migration gap (suppress pattern) documented;
  §2.3 value type clarified. P2 and P6 marked moot. All open questions closed.
- v1.1 (2026-03-23): §2 expanded with Option A/B comparison; §3.3.1 separator
  analysis; Q3 mode resolution confirmed.
- v1.0 (2026-03-23): Initial Draft.
