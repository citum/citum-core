# Citum Design Principles

**Status:** Active architectural doctrine
**Scope:** Project-wide design constraints for Citum schema, styles, migration,
engine behavior, and public surfaces.

This document states the durable principles. Operational policy lives in
`CLAUDE.md`; detailed contracts live in the linked specs and policies.

## 1. Fidelity Is The First Constraint

Citum is not a new citation aesthetic. It is a typed, declarative citation
system whose first obligation is to render known styles correctly.

- **Declared authority comes first.** When a current style guide, publisher
  instruction, manual, or curated documentary example states the rule, Citum
  follows that authority even if CSL XML, citeproc-js output, or migrated YAML
  says something easier to test.
- **Fidelity checks are the automation gate.** CSL-derived styles are usually
  compared against citeproc-js for supported features; biblatex-derived work is
  compared against biblatex behavior where that is the relevant prior art.
  These comparators are executable authorities or fallbacks, not a replacement
  for better documentary evidence.
- **Byte-visible output matters.** Punctuation, spacing, affixes, ordering,
  disambiguation, and locale terms are architectural behavior, not cosmetic
  cleanup.
- **SQI is secondary.** Style Quality Index can guide concision, preset reuse,
  and fallback robustness, but no SQI gain justifies a fidelity regression.
- **Divergence must be explicit.** Intentional departures from CSL/citeproc or
  biblatex behavior belong in the divergence register or a focused spec, not
  in hidden processor behavior.

See also: [`../TIER_STATUS.md`](../TIER_STATUS.md),
[`../policies/SQI_REFINEMENT_PLAN.md`](../policies/SQI_REFINEMENT_PLAN.md),
[`../adjudication/DIVERGENCE_REGISTER.md`](../adjudication/DIVERGENCE_REGISTER.md),
[`../policies/STYLE_WORKFLOW_DECISION_RULES.md`](../policies/STYLE_WORKFLOW_DECISION_RULES.md),
[`MULTI_AUTHORITY_STYLE_VERIFICATION_PLAN_2026-03-07.md`](./MULTI_AUTHORITY_STYLE_VERIFICATION_PLAN_2026-03-07.md).

## 2. Data Model Before Formatting Tricks

The reference model should preserve bibliographic reality before it serves a
particular output string.

- **EDTF-first dates.** Date fields prefer parsed EDTF so ranges,
  uncertainty, approximation, seasons, and open intervals remain structured.
  Literal fallback is allowed when data cannot be parsed without loss.
- **Structured contributors.** Personal names, organizational names, and
  contributor lists are distinct inputs. Processors must not parse display
  strings to recover name structure; corporate names may contain commas.
- **Hybrid reference classes.** Academic parent-child relationships are
  structural when the parent is semantically meaningful, such as journal
  articles and book chapters. Domain objects are flat when the apparent
  container is only a locator, such as legal reporters, treaty series,
  repositories, and standards bodies.
- **New top-level types require evidence.** Add a class only when semantic
  distinction, style discrimination, field-schema difference, and lack of a
  meaningful parent all justify it.
- **Rich text is bounded.** Freeform text fields support plain strings and Djot
  inline markup. Full executable or format-specific fragments are not part of
  the portable data model.

See also: [`../policies/TYPE_ADDITION_POLICY.md`](../policies/TYPE_ADDITION_POLICY.md),
[`../specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md`](../specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md).

## 3. Multilingual Data Is First-Class

Multilingual and multiscript behavior is data and style configuration, not an
after-the-fact rendering patch.

- **Use explicit language/script tags.** Language identifiers use BCP 47 style
  tags. Reference data may carry item language, per-field language overrides,
  original text, transliterations, and translations.
- **Render the requested view when possible.** Styles declare multilingual
  modes such as primary, transliterated, translated, and combined. Missing
  preferred views degrade to available data rather than failing.
- **Disambiguate what the reader sees.** Name and title disambiguation operates
  on displayed written forms. Identifiers such as DOI and ORCID establish
  identity; they do not replace surface-form disambiguation.
- **Locale and script choices are declarative.** Locale terms, grammatical
  forms, transliteration priority, and script-specific behavior belong in style
  or locale data.

See also: [`MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md`](../specs/MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md),
[`DISAMBIGUATION_MULTILINGUAL_GROUPING.md`](./DISAMBIGUATION_MULTILINGUAL_GROUPING.md).

## 4. Styles Are Declarative Contracts

The style language should describe citation behavior directly. The processor
should execute the contract, not infer house rules from hidden type checks.

- **Explicit over magic.** If journals, books, legal cases, or datasets need
  different punctuation or components, the style declares that behavior.
- **Templates replace procedural conditionals.** Citum templates and options
  replace CSL 1.0 macro/choose/if structures with typed, inspectable data.
- **Generic spines first.** Prefer lean base templates, option presets, template
  presets, and inheritance before adding per-type variants.
- **Type variants are for structural outliers.** Use `type-variants` only when
  a reference class or subtype genuinely needs a different component set or
  ordering.
- **Reuse is part of the language.** `extends`, `extends-pin`, template refs,
  embedded bases, option presets, and null-aware overlays are architectural
  mechanisms for keeping styles small without hiding semantics.

Example of hidden processor magic to avoid:

```rust
if ref_type == "article-journal" {
    separator = ", ";
}
```

The same distinction belongs in style data:

```yaml
bibliography:
  type-variants:
    article-journal:
      template:
        - title: parent-serial
          suffix: ","
```

See also: [`MIGRATION_STRATEGY_ANALYSIS.md`](./MIGRATION_STRATEGY_ANALYSIS.md),
[`CITUM_MODULARIZATION.md`](./CITUM_MODULARIZATION.md).

## 5. Compatibility Is A Public Contract

Citum is pre-1.0, but style and data producers still need predictable upgrade
behavior.

- **Rust types are schema truth.** Serde structs/enums define the data model;
  JSON Schema is generated from code and checked into docs when schema crates
  or the CLI schema generator change.
- **Older engines should degrade deliberately.** Additive style/data/locale
  changes normally produce `SoftDegrade` with warnings, not raw serde errors.
- **Template grammar changes are different.** New required template variants,
  required fields, or changed component semantics are major-level changes and
  may hard-fail in older engines.
- **Unknown reference classes round-trip.** The `InputReference` discriminator
  captures unknown top-level classes and lets document APIs warn while
  preserving data.
- **Warnings are API behavior.** Compatibility warnings must surface through
  documented CLI, engine, server, and binding channels rather than ad hoc
  logging.

See also: [`../specs/FORWARD_COMPATIBILITY.md`](../specs/FORWARD_COMPATIBILITY.md),
[`../policies/ENUM_VOCABULARY_POLICY.md`](../policies/ENUM_VOCABULARY_POLICY.md),
[`../reference/SCHEMA_VERSIONING.md`](../reference/SCHEMA_VERSIONING.md).

## 6. Migration Is Hybrid By Design

CSL 1.0 and Citum have different shapes. The migration strategy must respect
that mismatch instead of pretending XML can be translated mechanically into a
clean declarative model.

- **Keep what XML is good at.** CSL XML remains useful for extracting options,
  locale terms, substitute rules, disambiguation hints, and latent branches.
- **Do not over-trust XML template compilation.** Procedural macro and
  conditional flattening is the hard part of migration; heuristic fixes should
  be treated as risk unless comparator or fixture evidence supports them.
- **Use output-driven evidence where it is stronger.** citeproc-js output can
  reveal actual component order, delimiters, formatting, and type-specific
  suppression for exercised reference types.
- **Hand-author high-impact styles when needed.** Production parent styles are
  allowed to be domain-authored and presetized rather than treated as compiler
  artifacts.
- **Migration success is measured downstream.** The converted style is not done
  when YAML parses; it is done when rendering, fallback behavior, and quality
  gates hold.

See also: [`MIGRATION_STRATEGY_ANALYSIS.md`](./MIGRATION_STRATEGY_ANALYSIS.md),
[`ROADMAP.md`](./ROADMAP.md).

## 7. One Engine, Multiple Surfaces

The core engine should remain a synchronous, deterministic library that can be
reached through several product surfaces.

- **Batch and interactive modes are both first-class.** The CLI supports
  document and reference workflows; `citum-server` provides long-running
  JSON-RPC over stdio and default-enabled HTTP for low-latency clients.
- **Rendering targets share semantics.** Plain text, HTML, Djot, LaTeX, and
  Typst renderers should differ in markup representation, not in citation
  logic.
- **Document processing is part of the system.** Citations, note behavior,
  bibliography placement, grouping, and output-format details are engine
  behavior, not CLI-only conveniences.
- **Bindings are consumers of the same contract.** Lua, Python, JS, FFI, server,
  and CLI surfaces should expose the same style/data semantics and warning
  behavior.

See also: [`../../crates/README.md`](../../crates/README.md).

## 8. Extensibility Protects Portability

Extensions exist to relieve pressure on the core schema without turning shared
styles into processor-dependent programs.

- **Promote repeated needs into the core schema.** Common behavior should become
  a typed field, option, preset, locale term, or template construct.
- **Use `custom.*` as inert metadata.** Namespaced custom data may incubate
  domain-specific information, but portable rendering must not silently depend
  on executable interpretation of it.
- **Prefer constrained declarative rules before scripting.** Any future rule
  layer must be deterministic, bounded, serializable, schema-visible, and safe
  to ignore or degrade.
- **Do not put executable code in portable shared styles.** Runtime scripting,
  host plugins, preprocessors, and postprocessors belong at local deployment
  boundaries unless a later spec explicitly says otherwise.

See also:
[`EXTENSIBILITY_STRATEGY.md`](./EXTENSIBILITY_STRATEGY.md).

## 9. Rust Engineering Serves The Model

Engineering standards are not ornamental. They protect schema truth,
determinism, and diagnosability.

- **No hidden panic paths in production logic.** Use `Result`, typed errors, and
  explicit fallbacks. `unwrap`/`expect` belong only in tests, benchmarks,
  checked invariants with local rationale, or fatal bootstrap paths.
- **No `unsafe` without separate justification.** If it ever becomes necessary,
  the invariant and boundary must be documented at the use site.
- **Comments explain constraints.** Good comments record why a design exists,
  what invariant the type system cannot express, or which external spec forced
  a behavior. They do not narrate obvious code.
- **Public Rust items are documented.** The schema is public surface; new or
  touched public items need concise `///` documentation.
- **Generated artifacts stay coupled to source.** Schema and docs generated from
  Rust must be regenerated in the same change that modifies their source.

See also: [`../guides/CODING_STANDARDS.md`](../guides/CODING_STANDARDS.md).
