# Citum Extensibility Strategy

**Status:** Accepted architectural direction
**Last updated:** 2026-05-17
**Related:** [DESIGN_PRINCIPLES.md](./DESIGN_PRINCIPLES.md), [PRIOR_ART.md](./PRIOR_ART.md),
[../specs/FORWARD_COMPATIBILITY.md](../specs/FORWARD_COMPATIBILITY.md)

## Problem Statement

CSL's long resistance to extensions was correct for the ecosystem it served.
Once styles can depend on implementation-specific behavior, the shared style
catalog stops being robust: debugging becomes harder, and authors must learn
processor quirks instead of a common language.

Citum can revisit that question in 2026 because its architecture is different:
the core schema is code-first, strict, and still evolving. But the original CSL
lesson still holds. Extensibility is only worth adding if it helps with rare
style behavior without turning published styles into runtime-dependent programs.

This document answers a narrower question than "should Citum have plugins?".
The question is: when new rendering or data needs arise, what is the right path
for them? It is an architecture decision about where new behavior belongs, not
a plugin design or executable-extension spec.

---

## Decision

Citum has two paths for new needs, not a graduated ladder:

1. **Rendering behavior goes into the core schema.** If something needs to
   appear in formatted output, it needs a typed field, enum variant, or
   template component. There is no intermediate rendering surface.

2. **Non-rendering metadata may use `custom.*` fields.** The existing
   `custom` escape hatch can carry inert metadata (a publisher's internal
   style ID, dataset provenance). The engine ignores these fields entirely.
   Most `custom` placements in the schema have no demonstrated use case
   and may be dead weight — see the honest assessment below.

Everything else — executable extensions, preprocessors, postprocessors,
runtime plugins — is future work that does not exist yet and is not part of
the style format today.

### Governing constraints

- Rendering behavior in shared styles is determined by the core schema and
  documented declarative features, nothing else.
- `custom.*` is inert metadata that the engine ignores. It is not a rendering
  surface and not an incubation path. Most `custom` placements in the schema
  have no demonstrated use case.
- No executable code belongs in shared style YAML. Styles are declarative
  data, not programs.

---

## Core Schema as the Rendering Path

If multiple important styles need the same behavior, the response is to add an
explicit declarative field, preset, or template component to the schema.

This keeps Citum aligned with its central rule: the style should be explicit
and the processor should stay dumb. New schema additions are:

- Typed and validated at parse time
- Testable against fixture comparisons
- Inspectable via JSON Schema
- Subject to the forward-compatibility contract: additive changes ship as a
  `minor` schema bump, and older engine builds degrade gracefully with
  warnings rather than hard errors (see
  [`FORWARD_COMPATIBILITY.md`](../specs/FORWARD_COMPATIBILITY.md))

This is the only path for rendering behavior. There is no shortcut through
`custom.*`, expressions, or plugins.

### When to add a schema feature

Use this sequence:

1. **Evidence of repeated need.** Multiple styles, multiple domains, or a
   single dominant style guide that clearly requires the behavior.
2. **Declarative representation.** The behavior can be expressed as typed data
   (field, enum variant, option, preset, template component) rather than
   arbitrary logic.
3. **Deterministic and bounded.** The addition produces the same output for
   the same input. No I/O, no host APIs, no randomness.

If a proposed addition requires non-determinism, host APIs, or external I/O,
it is disqualified from the style format.

---

## `custom.*` Fields: Current State and Honest Assessment

The schema currently places `custom: Option<HashMap<String, serde_json::Value>>`
on many surfaces. The engine never reads or acts on any of them during
rendering. They parse, serialize, and round-trip silently.

### Where `custom` fields exist today

| Surface | Concrete use case |
|---|---|
| `Style` (top-level) | Thin — perhaps a publisher's internal style identifier |
| `BibliographySpec`, `CitationSpec` | None identified |
| Every template component (`TemplateContributor`, `TemplateDate`, etc.) | None identified |
| Option structs (`BibliographyOptions`, `DateOptions`, etc.) | None identified |
| `InputBibliography` (dataset level) | Thin — dataset provenance or source-system metadata |

Individual `InputReference` types do **not** have `custom` fields.

### The dead-weight question

Most of these placements have no concrete use case. `custom` on a
`TemplateContributor` or a `DateOptions` struct answers a question nobody
has asked. These fields were added speculatively as an escape hatch, but
the escape hatch leads nowhere — custom data cannot be rendered, so there
is nothing for a style author to escape _to_.

The placements that might carry some weight:

- **`Style` top-level:** a publisher or style registry could attach workflow
  metadata (internal ID, categorization tags, generation toolchain version).
- **`InputBibliography`:** a dataset could carry provenance information
  (source system, export timestamp, curator identifier).

Everything else is schema surface area with no demonstrated purpose. A future
cleanup could remove `custom` from template components and option structs
without losing any real capability.

### What `custom.*` is not

- **Not a rendering surface.** The engine does not render custom data.
- **Not an incubation path for rendering behavior.** There is no feedback
  loop from "store data in `custom.*`" to "see it in output." If domain data
  needs to be rendered, it needs a core schema addition.
- **Not a way to silently alter rendering semantics.** Shared styles must not
  rely on engines interpreting `custom.*` values to produce correct output.

### Namespacing rules

If `custom` fields are used at all:

- Use clear namespacing: `custom.publisher-x.*` or another documented
  vendor/domain prefix.
- Engines should ignore unrecognized `custom.*` keys rather than treating
  them as errors.

---

## No Executable Code in Shared Styles

Shared style YAML must not contain executable code. This includes embedded
scripting languages, expression evaluators, or references to runtime-loaded
code modules.

### Why

- The shared style catalog is the primary coordination mechanism. Styles
  that require runtime code to produce correct output break the catalog
  contract.
- Schema validation would stop being the full contract for correct rendering.
- Debugging and long-term maintenance would shift from style design to
  runtime extension behavior.

### Steel, WASM, and plugin systems

Steel (a Rust-hosted Scheme runtime) and WASM are interesting embedding
technologies. They are not style language features. If Citum ever builds
preprocessing, postprocessing, or plugin hooks around the engine, that is
separate architecture that does not exist yet and would need its own spec.

The important distinction: runtime hooks around the engine are a different
question from putting code inside style YAML. The former might eventually be
useful for institutional or research workflows. The latter is rejected.

---

## Relationship to Forward Compatibility

The [`FORWARD_COMPATIBILITY`](../specs/FORWARD_COMPATIBILITY.md) spec
classifies how older engine builds handle newer data:

| Category | Outcome | Meaning |
|---|---|---|
| `custom.*` fields | `Pass` | Silently accepted, no rendering effect |
| Core schema additions (new enum variant, new option key, new field) | `SoftDegrade` | Acknowledged with warning, rendering continues without the new feature |
| Template grammar changes | `HardFail` | Parse error, no render (major-version territory) |

The promotion path for rendering behavior is:

1. A typed schema addition is designed when evidence supports it. This is the
   rendering debut — no rendering happens before this point.
2. The schema addition ships as a `minor` bump. Older engines see it as
   `SoftDegrade`.

`custom.*` is not part of this promotion path. It is a separate,
non-rendering metadata surface.

---

## Decision Rules

When a new need appears:

1. **Does it need to appear in formatted output?** → Core schema addition.
   Design the typed field, variant, or template component.
2. **Neither rendering nor schema-relevant?** → Not supported by the style
   format today.

---

## Worked Scenarios

### Rare style edge case

A top style needs one unusual formatting switch for a narrow condition.

Recommended evaluation:
- If other major styles likely share the need, add a schema field or option.
- If the need is genuinely unique to one style, consider whether a modest
  schema addition (an enum variant, an option flag) is justified.
- `custom.*` does not help here — the behavior needs to produce output.

### Domain-specific legal behavior

Legal citation may need jurisdiction-aware logic, parallel citations, or other
domain structures not yet in the core type system.

Recommended evaluation:
- Treat this as a domain design problem: what fields, types, and template
  components does the schema need?
- Do not try to incubate rendering behavior in `custom.*`. Legal citation
  data stored in `custom.citum-law.*` is invisible to the renderer.
- Promote stable recurring concepts into the core schema with typed fields.

### Research prototype

A researcher wants to experiment with novel citation transforms or
institution-specific rules.

Recommended evaluation:
- The style format does not support this today.
- If Citum ever builds engine-level hooks or plugin infrastructure, that
  would be the place for experimental transforms — but that infrastructure
  does not exist yet and would need its own spec.

---

## Non-Decision Boundaries

This document does not:

- Add a new schema feature
- Change current rendering behavior
- Define a plugin API or hook system
- Bless any scripting language as part of style YAML
- Design a constrained expression or rule language

Any of those would require a separate spec in `docs/specs/`.

---

## Author Decision Checklist

- Can this be expressed using the current core schema or a modest schema
  addition?
- If not, is it repeated and important enough to justify core-schema design
  work?
- If it does not need to produce visible output, is it simply structured
  metadata that belongs in a namespaced `custom` field?
- If none of the above apply, is this genuinely out of scope for the style
  format today?

---

## Acceptance Checks

- A rare style edge case can be evaluated as a core-schema addition rather
  than defaulting to scripting or `custom.*`.
- Domain-heavy areas such as legal citation are understood as schema design
  problems, not extension or plugin problems.
- The recommendation remains consistent with
  [DESIGN_PRINCIPLES.md](./DESIGN_PRINCIPLES.md), especially explicitness,
  strict validation, and declarative contracts.
