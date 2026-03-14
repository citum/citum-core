# Citum Extensibility Strategy

**Status:** Accepted architectural direction
**Last updated:** 2026-03-14
**Related:** [DESIGN_PRINCIPLES.md](./DESIGN_PRINCIPLES.md), [PRIOR_ART.md](./PRIOR_ART.md), [../guides/DOCUMENT_CLASSIFICATION.md](../guides/DOCUMENT_CLASSIFICATION.md)

## Problem Statement

CSL's long resistance to extensions was correct for the ecosystem it served.
Once styles can depend on implementation-specific behavior, the shared style
catalog stops being robust: portability degrades, debugging becomes harder, and
authors must learn processor quirks instead of a common language.

Citum can revisit that question in 2026 because its architecture is different:
the core schema is code-first, strict, and still evolving. But the original CSL
lesson still holds. Extensibility is only worth adding if it helps with rare
style behavior without turning published styles into runtime-dependent programs.

This document answers a narrower question than "should Citum have plugins?".
The question is: how should Citum handle edge-case behavior that does not yet
fit the declarative core, while preserving a portable style ecosystem? It is an
architecture decision about extensibility governance and non-portable behavior
boundaries, not a plugin design or executable-extension spec.

---

## Decision

Yes, Citum should reconsider extensions, but only as a tightly constrained
escape hatch. The portable declarative core remains primary. Extensions are a
fallback for rare cases, not a second programming model.

The recommended strategy is an ordered extension ladder:

### Governing constraints

- Portable shared-style behavior should be determined by the core schema and
  documented declarative features.
- `custom` is an incubation surface for data-only metadata, not a backdoor for
  silent semantic overrides.
- Executable behavior is not part of the portable style format today.

### 1. Promote common needs into the core schema first

If multiple important styles need the same behavior, the default response is to
add an explicit declarative field, preset, or override shape to the schema.

This keeps Citum aligned with its central rule: the style should be explicit
and the processor should stay dumb.

### 2. Use namespaced data-only extensions second

If a need is real but not yet mature enough for the portable schema, it may use
the existing `custom` fields as inert metadata only.

Rules:
- Use clear namespacing such as `custom.citum-law.*`, `custom.publisher-x.*`,
  or another documented vendor/domain prefix.
- These fields must not silently change baseline rendering semantics in normal
  portable execution.
- Shared styles may carry such metadata, but interoperable rendering must not
  depend on executable interpretation of it.
- Portable engines should ignore unrecognized `custom.*` keys rather than
  treating them as errors in a shared style.

Current interpretation: existing `custom` support remains data-only, and no
executable extension surface is standardized yet.

Examples:
- Good: `custom.citum-law.parallel-citations` carrying structured legal metadata
  that a specialized renderer may optionally use.
- Bad: `custom.publisher-x.suppress-author-if-local` causing a house renderer
  to silently alter shared-style baseline semantics.

### 3. Prefer a tiny declarative expression layer before full scripting

If rare style rules truly need limited computed behavior, Citum should evaluate
a constrained rule or expression layer before adopting a general-purpose
runtime.

Candidate scope:
- field presence tests
- simple normalization or fallback selection
- formatting switches
- small derived values that are deterministic and serializable

Requirements:
- deterministic
- auditable
- representable in schema and generated JSON Schema
- testable against oracle fixtures
- safe to ignore or degrade in older engines
- no I/O, network access, time access, or randomness
- no recursion, unbounded loops, user-defined functions, or dynamic code loading

Any future expression support must have bounded evaluation and deterministic
results for finite inputs. Exact opcode and fallback semantics are future spec
work, not something this ADR settles.

This is the highest rung that could plausibly become part of the portable style
format. If it is ever pursued, it needs a separate spec in `docs/specs/`.

### 4. Do not put embedded Lisp or Steel in the portable style format now

Embedded Lisp remains an interesting idea, and Steel is a credible example of a
Rust-hosted Lisp runtime with attractive embedding ergonomics. But that is not
the right first extension surface for Citum styles.

Reasons:
- it would create a second authoring language beside YAML
- style portability would become environment-dependent
- schema validation would stop being the full contract
- oracle testing would have to account for runtime code paths, not just style data
- debugging and long-term maintenance would shift from style design to runtime behavior

Steel is therefore worth keeping in view as a future local engine or plugin
mechanism, but it is explicitly rejected as the first portable extension layer.
The promising use cases are local institutional transforms, legal-citation
experiments, renderer-side postprocessing, and research prototypes.

### 5. Reserve full runtime scripting for non-portable local adapters

If Citum ever adds scripting, it should initially live at the application or
deployment boundary:
- local preprocess hooks
- local postprocess hooks
- app-side adapters
- experimental research workflows

That allows experimentation without polluting the shared style catalog. A style
published for general reuse should not require arbitrary embedded code. Local
hooks are allowed only outside the portable style contract and must not be
required for correct interpretation of a shared style file. If such hooks ever
exist, they belong in host configuration or host code rather than ordinary
shared style YAML.

---

## Why Reconsider Extensions At All

The right 2010s answer for CSL was often "no extensions". The ecosystem value
came from one common style language shared across many processors and tools.
That remains a hard-earned lesson.

The 2026 case for reconsideration is narrower:
- Citum is still defining a modern declarative schema rather than inheriting
  the exact limits of CSL 1.0.
- Some edge cases may not justify immediate schema growth but still deserve a
  place to live.
- Citum already has explicit `custom` metadata slots, so the architecture has a
  bounded place for non-core information.
- Domain-heavy work such as legal citation may need staged incubation before a
  stable portable model is clear.

Reconsidering extensions does not mean relaxing portability standards. It means
choosing an escalation path that protects them.

---

## Option Comparison

| Option | Ecosystem portability | Author comprehensibility | Runtime safety | Testability / oracle fit | Migration pressure | Maintenance burden |
|---|---|---|---|---|---|---|
| No extensions at all | Highest | High | Highest | Highest | High, because every new need pressures the core immediately | Medium |
| Core-schema-only growth | High | High | High | High | Medium to high | Medium |
| Namespaced inert metadata | High | High | High | High | Low | Low |
| Constrained rule / expression DSL | Medium to high if tightly scoped | Medium | High if deterministic | Medium to high | Low to medium | Medium to high |
| Embedded Lisp / Steel | Low for shared styles | Low to medium | Medium, depending on sandboxing | Medium at best | Low short-term | High |
| WASM or external plugin hooks | Low for shared styles | Low | Medium | Medium | Low short-term | High |
| External preprocess / postprocess pipeline | Medium for deployments, low for published style portability | Medium | Medium to high | Medium | Low | Medium |

Interpretation:
- `No extensions at all` is safest but pushes every edge case into either core
  growth or unsupported behavior.
- `Core-schema-only growth` should remain the default path for repeated needs.
- `Namespaced inert metadata` is the safest intermediate layer and should be
  used now.
- A tiny expression layer is the only executable direction that is plausibly
  compatible with portable styles, but only if the scope stays very small.
- Successful recurring patterns should move upward into the core schema over
  time rather than remaining permanent extension debt.
- Steel, WASM, and plugin hooks are best understood as local runtime tools, not
  shared style-language features.

---

## Decision Rules

When a new need appears, use this sequence:

1. If several important styles need it, promote it into the core schema.
2. If it is still exploratory or domain-specific, model it as namespaced
   data-only metadata first.
3. If data-only metadata is insufficient, ask whether a tiny declarative rule
   can solve it.
4. Only after those options fail should local scripting or plugin mechanisms be
   considered, and then only outside the portable style contract.

Pressure should flow upward: experiments may graduate from `custom` to a
constrained expression layer to the core schema, but stable core behavior
should not be pushed back down into local plugins.

Kill switch: if a proposed portable feature requires non-determinism, host
APIs, or external I/O, it is disqualified from the portable format and belongs
in local scripting or adapters only.

This keeps extension pressure visible and creates a promotion path from
experiment to portable feature.

---

## Worked Scenarios

### Rare style edge case

A top style needs one unusual formatting switch for a narrow condition.

Recommended evaluation:
- if other major styles likely share the need, add a schema field
- otherwise, carry the metadata in a namespaced `custom` slot while evidence accumulates
- only consider a tiny expression layer if the behavior cannot be represented declaratively

### Domain-specific legal behavior

Legal citation may need jurisdiction-aware logic, parallel citations, or other
domain structures not yet stabilized in the core type system.

Recommended evaluation:
- treat this first as a domain design problem
- promote stable recurring concepts into the core schema
- avoid jumping directly to scripting just because the domain is complex

### Local research prototype

A local user wants to experiment with novel transforms or institution-specific
rules.

Recommended evaluation:
- allow this, if at all, as a local adapter or runtime hook
- do not require shared styles to embed executable code for the feature

---

## Non-Decision Boundaries

This document does not standardize a new executable extension system.

It explicitly does not:
- add a new schema feature
- change current rendering behavior
- bless embedded Lisp as part of ordinary style files
- define a plugin API
- replace the need for a future spec if executable extensions are pursued

---

## Author Decision Checklist

- Can this be expressed using the current core schema or a modest schema
  addition?
- If not, is it portable and repeated enough to justify core-schema design work?
- If not, is it simply structured metadata that belongs in a namespaced
  `custom` field?
- If code appears necessary, is this actually local-adapter territory rather
  than portable style-language behavior?

---

## Acceptance Checks

- A rare style edge case can be evaluated as either a core-schema addition or a
  namespaced extension, rather than defaulting to scripting.
- Domain-heavy areas such as legal citation can incubate without forcing a
  general plugin model.
- A local research prototype can use non-portable hooks without polluting the
  shared style ecosystem.
- The recommendation remains consistent with
  [DESIGN_PRINCIPLES.md](./DESIGN_PRINCIPLES.md), especially explicitness,
  strict validation, and portability.
- Steel is documented as an interesting future local-runtime option, but not as
  the default extension strategy for the portable style layer.
