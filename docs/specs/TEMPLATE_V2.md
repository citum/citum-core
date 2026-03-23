# Template Schema v2 Specification

**Status:** Draft
**Version:** 1.0
**Date:** 2026-03-23
**Supersedes:** (none)
**Related:** csl26-da9f

## Purpose

This spec defines a focused set of improvements to the Citum template schema.
The changes address three concrete authoring pain points — a misleading key
name (`items`), a fragmented type-override mechanism (`overrides`), and verbose
nested YAML — plus a catalog of additional pain points discovered during a
full schema audit.

§2 presents two options for `overrides`: full removal (preferred) or legacy
retention with a rename. The decision is open pending user confirmation.

## Scope

**In scope:**
- Rename the `items` YAML key to `group` (struct rename + serde alias).
- Remove or retire per-component `overrides` (§2 presents both options).
- Promote `type-variants` at the citation and bibliography spec level.
- Design a compact string template DSL for GUI text-field authoring.
- Catalog additional pain points with effort estimates and v2 scope decisions.

**Out of scope:**
- Changing `TemplateComponent` discrimination (untagged serde remains).
- Altering CSL group-suppression semantics.
- Implementing the compact DSL parser in this spec cycle (design only).
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

### §2 — Per-Component `overrides` and `type-variants`

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

#### 2.2 Problems with `overrides`

1. **Scanning cost.** A reader must visit every component to understand what a
   given type renders. A GUI must re-parse the full template tree per type.
2. **No top-level view.** There is no single place that shows "here is what a
   `personal-communication` renders differently."
3. **Semantic ambiguity.** `ComponentOverride::Component(Box<TemplateComponent>)`
   allows replacing a `contributor` component with a `date`. Semantically wrong,
   but parses silently.
4. **GUI hostility.** Any GUI that wants to show a per-type preview must traverse
   the entire component tree, merge overrides, and reconstruct a virtual template.
   There is no clean data model to bind to a tab panel.
5. **Compact DSL incompatibility.** The compact string syntax (§3) cannot express
   `overrides` — they are a sub-object of arbitrary depth. A component with
   overrides cannot be expressed in compact form at all, even if the component
   itself is simple. This breaks the authoring story for any style that currently
   uses overrides for minor per-type tweaks.

#### 2.3 Option A — Full Removal (Preferred)

Remove `overrides` from `TemplateComponent` entirely. All type-specific behavior
moves to `type-variants` at the citation/bibliography spec level.

**What this requires:**

- Every use of per-component `overrides` in existing styles must be lifted into a
  `type-variants` block. The migration compiler does this today for most cases via
  `compile_bibliography_with_types`.
- A handful of rendering-only patches (e.g., `emph: false` for one type) that are
  currently inline in a component become a full duplicate template entry in
  `type-variants`. The duplication is real but bounded: only the components that
  differ need to appear in the variant template.
- The `ComponentOverride` enum and the `overrides` field on every
  `TemplateComponent` variant are removed.
- `citum-migrate` no longer emits `overrides` in its output; all type-branching
  emits as `type-variants`.
- Existing YAML styles that use `overrides` fail to parse after the change — no
  alias strategy can cover a removed field. A bulk migration pass is required.

**Why this is preferred:**

- `type-variants` is the clean model: one place per type, easy to tab, easy to
  hide ("Suppress for this type" toggle).
- The compact DSL (§3) works for any template that does not use `overrides`. Full
  removal means the compact DSL works for any template, period.
- Removes `ComponentOverride::Component(Box<TemplateComponent>)` — eliminates the
  cross-kind replacement footgun.
- Rust: fewer branches in the engine's dispatch paths; no per-component override
  lookup at render time.

**Duplication cost in practice (APA survey):**

APA uses `overrides` on approximately 15 components across bibliography and
citation templates. Most are structural divergences (legal-case, personal-
communication, patent, webpage) that would become `type-variants` entries anyway.
A small number are single-field rendering patches (e.g., `suppress: true` for
one type). Those become empty `type-variants` entries (`[]`) or slightly expanded
component lists — a one-time authoring cost, not ongoing.

#### 2.4 Option B — Legacy Retention with Rename (Conservative)

Keep `overrides` in the schema but designate it legacy. Rename to `type-overrides`
with `#[serde(alias = "overrides")]` for backward compat.

**Tradeoffs vs. Option A:**

| | Option A (Remove) | Option B (Retain) |
|---|---|---|
| GUI complexity | Low — type-variants only | High — must handle both |
| Compact DSL coverage | 100% of templates | Blocked wherever overrides exist |
| Engine complexity | Lower | Unchanged |
| Migration effort | Higher (bulk pass) | Lower (alias covers old styles) |
| Schema cleanliness | Clean | Two overlapping mechanisms forever |
| Footgun (cross-kind override) | Eliminated | Remains |

Option B is viable if bulk migration is not acceptable in the v2 timeframe. It
is not recommended for a v2 that also introduces the compact DSL, because the two
features are in direct conflict: overrides block compact authoring, and retaining
them means the GUI must forever support two code paths.

**Decision: open.** Confirm Option A or B before implementation begins.

---

#### 2.5 `type-variants` at the Spec Level (both options)

Regardless of the overrides decision, `type-variants` is added to `CitationSpec`
and `BibliographySpec.type-templates` is renamed:

| Spec | Before | After |
|------|--------|-------|
| `BibliographySpec` | `type_templates` | `type-variants` (alias on old key) |
| `CitationSpec` | (absent) | `type-variants` (new) |

`type-variants` maps a `TypeSelector` key to:
- A full `Template` (replaces the default template for that type), or
- `[]` (suppress entirely for that type).

Runtime: if the reference type matches a `type-variants` key, use that template;
otherwise fall through to the default `template`.

**Mode resolution for citations:** type-variant lookup happens after
`integral`/`non-integral` mode resolution. A type-variant on the mode-specific
sub-spec (e.g., `citation.integral.type-variants`) takes precedence over one on
the top-level `citation.type-variants` when mode is integral.

```yaml
bibliography:
  type-variants:
    personal-communication: []
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
    ...

citation:
  type-variants:
    legal-case:
      - title: primary
        emph: true
        quote: false
      - date: issued
        form: year
        wrap: parentheses
        prefix: " "
  template:
    - contributor: author
      form: short
    - date: issued
      form: year
    - variable: locator
```

#### 2.6 Before/After: APA Personal Communication

**Before** (overrides scattered on two components):
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

**After** (single type-variant block):
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

#### 2.7 Before/After: APA Legal Case

**Before** (overrides on three components):
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

**After** (one type-variant entry, default template unchanged):
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

#### 2.8 Serde Rename: `type-templates` → `type-variants`

```rust
// BibliographySpec
#[serde(alias = "type-templates")]
pub type_variants: Option<HashMap<TypeSelector, Template>>,
```

#### 2.9 `citum-migrate` Impact

- `compile_bibliography_with_types` already emits `HashMap<TypeSelector, Template>`.
  Field rename to `type_variants` only.
- If Option A: `compile_for_type` in `compilation.rs` stops emitting `overrides`
  entirely; all type-branching emits as `type-variants` including at the citation level.
- If Option B: existing override emission is kept; citation-level `type-variants`
  is added for structural divergences only.

---

### §3 — Compact String Template DSL

#### 3.1 Problem

The full YAML template representation for even a simple citation pattern spans
10–20 lines. This creates friction in two scenarios:

1. **GUI text boxes.** A citum-hub wizard showing a single-line template field
   cannot accept raw YAML. A user who wants to type a quick pattern needs a
   compact notation.
2. **Spec examples and documentation.** Showing template patterns inline in
   prose requires multi-line YAML blocks; a compact form enables concise
   examples.

#### 3.2 Design Goals

- **Lossless / structural round-trip.** The compact string expands to an
  identical YAML AST. No information is dropped.
- **Human-writable.** A style author should be able to type a compact string
  in a GUI text box without reading a formal grammar.
- **Parser lives at the schema layer.** Compact strings are parsed into
  `Vec<TemplateComponent>` in `citum-schema-style`, not in the engine. The
  engine's input interface is unchanged.
- **Explicit about limits.** Some patterns require full YAML; the DSL does not
  attempt to cover them.

#### 3.3 Proposed Mini-DSL Syntax

Compact templates are a sequence of **component tokens** separated by a
component separator (see §3.3.1).

```
<component-token> ::= <kind>:<value>[/<modifier>]* [+<rendering>]*
<kind>           ::= contributor | date | title | number | variable | term | group
<modifier>       ::= secondary field (form, wrap, name-order, etc.)
<rendering>      ::= prefix:<str> | suffix:<str> | emph | quote | suppress
```

- **Colon** (`:`) separates kind from value within a token.
- **Slash** (`/`) introduces modifiers within a component.
- **Plus** (`+`) introduces rendering hints.
- **Groups** use `group(...)` with comma-separated interior.

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

**Groups** use `group(...)` with a comma-separated interior:

```
group(date:issued/year, variable:locator)+wrap:parentheses+delimiter:", "
```

#### 3.3.1 Component Separator Choice

The separator between component tokens is the most human-visible choice in the
DSL. Priority: legibility and ease of authoring. Three candidates:

**Option S1 — Pipe with spaces (` | `)**

```
contributor:author/short | date:issued/year+wrap:parentheses | variable:locator
```

Pros: high visual contrast; `|` is not a legal character in citation output, so
there is no collision risk except inside prefix/suffix strings.
Cons: `|` inside a prefix/suffix string (e.g., `prefix:"a|b"`) requires
backslash-escape. Rare in practice but a real edge case. Also a two-character
sequence (space-pipe-space) adds noise for short templates.

**Option S2 — Semicolon with spaces (` ; `)**

```
contributor:author/short ; date:issued/year+wrap:parentheses ; variable:locator
```

Pros: semicolon has a natural "separator" reading for most people; appears in
citation output only as a multi-cite separator (not inside a single-reference
template, so zero collision risk). No escape needed in practice.
Cons: slightly less visually prominent than `|` when components are long.

**Option S3 — Double-colon (`::`)**

```
contributor:author/short :: date:issued/year+wrap:parentheses :: variable:locator
```

Pros: zero collision risk even in strings; visually distinct from the single
`:` used within a component token.
Cons: reduces the legibility of individual tokens because `:` already appears
frequently (`kind:value`, `prefix:". "`); `::` as separator inside a line
of mostly `:` punctuation is harder to scan.

**Recommendation: S2 (semicolon).** Semicolons never appear in citation output
at the component-separator level, require no escape, and have an intuitive
"list of items" reading. The visual prominence is slightly lower than `|` but
the escape-free authoring experience is the higher priority for a human-facing
DSL. If the group interior also uses `,` as separator, a style guide note is
needed to explain that `,` (within a group) and `;` (between top-level
components) are distinct levels.

**Decision: open.** Confirm S1, S2, S3, or propose another before the parser
is implemented.

#### 3.4 Concrete Examples

**Example 1 — Simple APA non-integral citation:**

Full YAML:
```yaml
- contributor: author
  form: short
- date: issued
  form: year
  wrap: parentheses
- variable: locator
```

Compact:
```
contributor:author/short | date:issued/year+wrap:parentheses | variable:locator
```

**Example 2 — Chicago author-date citation:**

Full YAML:
```yaml
- contributor: author
  form: short
- date: issued
  form: year
- variable: locator
  prefix: ", "
```

Compact:
```
contributor:author/short | date:issued/year | variable:locator+prefix:", "
```

Note: `shorten` (a sub-object on the contributor) cannot be expressed in
compact form.

**Example 3 — Retrieved-from group:**

Full YAML:
```yaml
- group:
    - date: accessed
      form: year-month-day
      prefix: "Retrieved "
      suffix: ", from"
    - variable: url
  delimiter: " "
  prefix: " "
```

Compact:
```
group(date:accessed/year-month-day+prefix:"Retrieved "+suffix:", from", variable:url)+delimiter:" "+prefix:" "
```

#### 3.5 Parser Location and Error Handling

The parser lives at:

```
crates/citum-schema-style/src/template_dsl.rs
```

Public API:

```rust
pub fn parse_compact_template(s: &str) -> Result<Vec<TemplateComponent>, TemplateDslError>
```

Error cases:
- `UnknownKind` — kind not in the allowed set.
- `UnknownModifier { kind, modifier }` — modifier not valid for that kind.
- `UnclosedGroup` — `group(` without matching `)`.
- `UnescapedPipe` — literal pipe inside a prefix/suffix string.
- `GroupDepthExceeded` — nesting beyond limit (e.g., 5 levels).

The parser is a separate API surface for GUI tools; it is not invoked during
normal YAML deserialization. A future `#[serde(try_from = "String")]` wrapper
(`CompactTemplate`) may be added to allow compact strings directly in YAML, but
that is deferred.

#### 3.6 What Cannot Be Expressed in Compact Form

The following require full YAML:

- `shorten` (contributor name shortening sub-object)
- `label` (role label configuration on contributor)
- `fallback` (fallback components on date)
- `type-overrides` (per-type overrides on any component)
- `locales` (localized template overrides)
- `disambiguate-only` (on title)
- `links` (link configuration)
- `and`, `delimiter`, `sort-separator` on contributor
- `name-form` override

A GUI should detect these fields and switch the component editor to full-YAML
mode rather than silently dropping them.

---

### §4 — Additional Pain Points (Audit)

The following pain points were found during a schema-wide review from the
perspective of all three Citum personas (style author, GUI builder, migration
engineer). Each entry includes an effort estimate and a v2 scope decision.

#### P1 — Stringly-Typed `TypeSelector` (v2: Yes)

**Problem.** `TypeSelector::Single(String)` and `TypeSelector::Multiple(Vec<String>)`
accept any string. A typo like `article_journal` (underscore) instead of
`article-journal` silently matches nothing. The underscore normalizer in
`matches()` helps, but other typos produce silent no-ops.

**Proposed fix.** Add `validate_type_name(s: &str) -> bool` checking against
the canonical CSL/Citum type set. Call from a custom `TypeSelector`
deserializer, emitting a descriptive warning (not error, for forward-compat).

**Effort:** Low.

---

#### P2 — Two Concepts Named "Type Override" (v2: Yes)

**Problem.** Per-component `overrides` (rendering patch or component
replacement) and spec-level `type-variants` (full template replacement) both
express "type-specific behavior" at different granularities. The naming gives
no hint of this distinction.

**Proposed fix.** Rename per-component `overrides` → `type-overrides` with
`#[serde(alias = "overrides")]`. Update all YAML examples and migration output
to use `type-overrides`.

**Effort:** Low (rename + alias).

---

#### P3 — `CitationSpec.options` Accepts Bibliography-Only Fields (v2: No)

**Problem.** Both `CitationSpec` and `BibliographySpec` use `Option<Config>`,
which includes `bibliography: Option<BibliographyConfig>` and
`notes: Option<NoteConfig>` — meaningless in a citation context. The field
is silently ignored.

**Proposed fix.** Introduce `CitationOptions` / `BibliographyOptions` structs
containing only applicable fields, with a shared `CommonOptions` base.

**Effort:** High. **Deferred.** Document applicable fields in schema doc
comments as a stopgap.

---

#### P4 — Duplicate Variables Across `SimpleVariable` and `NumberVariable` (v2: No)

**Problem.** `Volume`, `Number`, `DocketNumber`, `PatentNumber`,
`StandardNumber`, `ReportNumber` appear in both `SimpleVariable` (for
`TemplateVariable`) and `NumberVariable` (for `TemplateNumber`). A style author
cannot predict which variant to use. The distinction (numeric formatting vs.
string passthrough) is not self-documenting.

**Proposed fix (A).** Remove duplicates from `SimpleVariable`; force `number:`
for all numerically-formattable fields. Migration pass required.

**Proposed fix (B).** Add a `format:` field to `TemplateVariable` for numeric
fields so both variants accept the same formatting options.

**Effort:** Medium–High. **Deferred.** Document the distinction in schema
comments.

---

#### P5 — `inner-prefix`/`inner-suffix` Not Tied to `wrap` (v2: No)

**Problem.** `Rendering.inner_prefix` and `Rendering.inner_suffix` only make
sense when `wrap` is set. Without `wrap`, they are silently ignored. A GUI
cannot enforce the dependency.

**Proposed fix.** Fold `inner-prefix` and `inner-suffix` into `WrapPunctuation`
as an optional struct variant (e.g., `Custom { open, close }` extended with
`inner_prefix`/`inner_suffix`).

**Effort:** Medium. Serde representation change; affects existing YAML.
**Deferred.**

---

#### P6 — `ComponentOverride::Component` Allows Cross-Kind Replacement (v2: No)

**Problem.** `ComponentOverride::Component(Box<TemplateComponent>)` allows
replacing a `contributor` component with a `date` override — changing the
component kind under a type override. This parses silently and produces
unexpected engine behavior.

**Proposed fix.** Restrict to same-kind replacements via per-kind override
structs. Breaking schema change.

**Effort:** High. **Deferred.** Add a semantic validation check in
`Style::validate()` as a non-breaking stopgap.

---

#### P7 — No `Style::validate()` Method (v2: Partial)

**Problem.** All validation happens at serde parse time or silently at render
time. GUI tools must run the full processor to discover authoring errors.

**Proposed fix.** Add `Style::validate(&self) -> Vec<SchemaWarning>` for
semantic checks: unrecognized type names in selectors, `inner-prefix` without
`wrap`, `type-variants` keys matching no known type, etc. Non-fatal warnings
preserve forward-compat.

**Effort:** Medium. **Partial v2 scope:** wire up `TypeSelector` validation
from P1 as the first implementation. Full semantic validation is deferred.

---

## Implementation Notes

### Ordering of Changes

Changes can land in this order to minimize churn:

1. `items` → `group` rename (template.rs only; alias covers compat).
2. `type-variants` addition to `CitationSpec` (new field; no breakage).
3. `type-templates` → `type-variants` rename on `BibliographySpec` (alias).
4. `overrides` → `type-overrides` rename on `TemplateComponent` variants (alias).
5. `TypeSelector` validation + `Style::validate()` stub (additive).
6. Compact DSL parser (`template_dsl.rs`) — standalone module, no schema breakage.

Step 6 is independent and lowest-risk; it can be developed in parallel.

### Schema Regeneration

After any `template.rs` or `lib.rs` change:

```bash
cargo run --bin citum --features schema -- schema --out-dir docs/schemas
git add docs/schemas/
```

Include a `Schema-Bump: patch` footer (new optional fields → patch).

### Migration Compiler

- `citum-migrate/src/template_compiler/bibliography.rs` —
  `compile_bibliography_with_types` already emits `HashMap<TypeSelector, Template>`.
  Update the output field name to `type_variants`.
- `template_compiler/compilation.rs` — extend to detect type-branching CSL
  `<choose>` at the citation level and emit `CitationSpec.type_variants`.

### Key File Locations

| Change | File |
|--------|------|
| `TemplateGroup`, `type-overrides` | `crates/citum-schema-style/src/template.rs` |
| `CitationSpec.type_variants`, `Style::validate` | `crates/citum-schema-style/src/lib.rs` |
| Compact DSL parser | `crates/citum-schema-style/src/template_dsl.rs` (new) |
| Migration output field | `crates/citum-migrate/src/template_compiler/bibliography.rs` |
| Migration citation type-variants | `crates/citum-migrate/src/template_compiler/compilation.rs` |

---

## Acceptance Criteria

- [ ] `- group:` and `- items:` both parse to `TemplateComponent::Group`.
- [ ] `TemplateGroup` serializes as `group:`, never `items:`.
- [ ] `BibliographySpec.type_variants` and `BibliographySpec.type_templates` both
      parse without error (alias).
- [ ] `CitationSpec.type_variants` accepts a `HashMap<TypeSelector, Template>`.
- [ ] (Option A) Per-component `overrides` field is absent from the schema; styles using it fail with a clear parse error pointing to `type-variants`.
- [ ] (Option B) Per-component `type-overrides` and `overrides` both parse without error (alias).
- [ ] `style/apa-7th.yaml` renders all 12 oracle scenarios without regression.
- [ ] `style/chicago-author-date.yaml` renders all 12 oracle scenarios without
      regression.
- [ ] `Style::validate()` emits a `SchemaWarning` for unrecognized type names in
      `TypeSelector`.
- [ ] `parse_compact_template("contributor:author/short | date:issued/year+wrap:parentheses")`
      returns a `Vec<TemplateComponent>` with two elements matching the expected
      YAML AST.
- [ ] `parse_compact_template("group(date:issued/year, variable:url)+prefix:\" \"")`
      returns a `TemplateGroup` with two children.
- [ ] `parse_compact_template("unknown:foo")` returns `Err(TemplateDslError::UnknownKind)`.
- [ ] Schema JSON regenerated and committed with a `Schema-Bump: patch` footer.
- [ ] All existing nextest suite passes without regression.

---

## Open Questions

1. **`overrides` — Option A (remove) or Option B (retain as legacy)?**
   Option A is preferred: cleaner schema, full compact DSL coverage, removes the
   GUI complexity of two overlapping type-override mechanisms. Requires a bulk
   migration pass on existing styles. Confirm before implementation begins.
   If Option A: P2 (`overrides` → `type-overrides` rename) is moot — the field
   is gone.

2. **Compact DSL component separator:** §3.3.1 recommends semicolon (` ; `).
   Alternatives are pipe (` | `, requires escape in strings) and double-colon
   (`::`, lower legibility). Confirm before the parser is implemented.

3. **`type-variants` mode resolution:** Confirmed — mode-specific sub-spec
   type-variants (e.g., `citation.integral.type-variants`) take precedence over
   top-level `citation.type-variants` when mode is integral.

---

## Changelog

- v1.1 (2026-03-23): §2 expanded with Option A (full removal) vs Option B (legacy retain) comparison; §3.3.1 added separator analysis with S1/S2/S3 candidates; mode resolution (Q3) confirmed.
- v1.0 (2026-03-23): Initial Draft.
