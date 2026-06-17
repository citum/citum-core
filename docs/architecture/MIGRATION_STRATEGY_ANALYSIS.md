# Migration Strategy Analysis: Hybrid Converter and Evidence Pipeline

## Current Position

`citum-migrate` is not the canonical authoring path for high-impact production
styles. It is a hybrid migration and evidence tool: it extracts durable
configuration from CSL 1.0 XML, resolves or synthesizes candidate templates,
scores candidates against oracle output, assembles a concrete Citum style, and
then refines the assembled result for Style Quality Index (SQI).

The strategic rule is the same as
[`DESIGN_PRINCIPLES.md`](./DESIGN_PRINCIPLES.md): fidelity is the first
constraint, SQI is secondary, and hand-authored or presetized parent styles are
valid when automatic migration reaches a quality plateau. Generated YAML is
acceptable only when downstream rendering and quality gates hold. A style that
parses is not done; a style that renders correctly but is unreadable is still
unfinished authoring work.

The older "XML compiler vs output-driven inference" framing remains useful, but
it is no longer the whole architecture. The current strategy is:

1. Use XML where XML is strongest: options, locale-ish signals, processing mode,
   contributor/date/title defaults, substitutes, and latent branches.
2. Use output-driven evidence where observed rendering is stronger than
   procedural flattening: component order, delimiters, formatting, and exercised
   type-specific suppression.
3. Use measured selection and held-out validation to choose between candidate
   styles rather than trusting one source unconditionally.
4. Use hand-authored or presetized styles for high-impact parents where clean
   style quality matters more than converter completeness.
5. Run SQI refinement after assembly, never inside the semantic assembly phase.

The SQI refinement split is the intended architecture. In checkouts that include
that split, assembly emits explicit templates and
`citum_migrate::passes::sqi_refinement` performs post-assembly diffing,
hoisting, pruning, and suppress cleanup. In older checkouts, read that boundary
as the target architecture rather than the implemented crate state. See also
[`SQI_REFINEMENT_PLAN.md`](../policies/SQI_REFINEMENT_PLAN.md) and
[`crates/README.md`](../../crates/README.md).

## Current `citum-migrate` Architecture

| Layer | Responsibility | Strategic status |
|---|---|---|
| XML parsing and macro expansion | Parse CSL 1.0, inline macros, preserve source provenance. | Necessary substrate. Good for semantic coverage, not sufficient for clean templates. |
| Option extraction | Extract contributor, date, title, bibliography, locator, and processing options. | Strongest part of the XML pipeline. Keep and extend conservatively. |
| XML compiler | Compile XML/IR into concrete citation and bibliography templates, including type templates. | Necessary fallback. Still risky for bibliography structure and type-conditioned macro flattening. |
| Template resolver and inferred templates | Load embedded, curated, or generated template candidates when they exist. | Preferred when curated/generated evidence is stronger than raw XML compilation. |
| Measured selection and synthesis | Score candidate citation/bibliography styles against citeproc-js fixtures and held-out validation. | Correct direction for bounded automatic improvement; limited by fixture coverage. |
| Semantic fixups | Apply fidelity-preserving full-template corrections such as URL/accessed gating and media/type repairs. | Acceptable only with oracle evidence or clear CSL/Citum semantic mismatch. Keep localized. |
| Assembly | Produce a standalone, explicit `citum_schema::Style` from selected sources. | Should be semantic and explicit. It must not hide differences through SQI compaction. |
| SQI refinement | Generate exact diff-based `type-variants`, hoist safe options, prune inherited defaults, normalize serialization noise. | Post-assembly concern. It improves maintainability only after the style is semantically assembled. |
| Lineage and wrappers | Attach `extends`, minimize wrappers, and emit evidence about standalone vs compressed forms. | Runs after standalone materialization so local diffs remain inspectable. |

This separation matters. If assembly strips fields, hoists options, or compacts
templates before diff generation, the diff engine compares already-mutated
templates and can produce diffs that reflect compaction artifacts rather than
semantic structure. Assembly should therefore prefer explicitness; SQI
refinement should own concision.

## Why Migrated Output Can Still Feel Wrong

Unsatisfying `citum-migrate` output usually falls into one of five buckets:

| Symptom | Meaning | Correct response |
|---|---|---|
| Rendered output differs from citeproc-js or a style guide. | Fidelity failure. | Fix converter, schema, or engine behavior with oracle/workflow evidence. No SQI tradeoff is allowed. |
| Rendered output is correct but YAML is verbose. | Structural ugliness, not necessarily semantic wrongness. | Improve SQI refinement, option hoisting, template presets, or hand-authored parent styles. |
| Type variants are numerous or unnatural. | Procedural CSL conditionals did not map cleanly to a declarative spine. | Prefer curated parent templates or output-driven evidence; keep XML fallback for latent behavior. |
| Behavior works only for exercised reference types. | Fixture coverage gap. | Expand selection and held-out fixtures before trusting output-driven inference. |
| Fixups feel style-specific or fragile. | Heuristic debt in media/type handling or bibliography flattening. | Narrow the fixup, add regression cases, or move the rule into schema/style data if it is general. |

This distinction is operationally important. A bad bibliography rendering is a
hard bug. A correct but bulky template is a style-quality problem. A clean YAML
file that only works for fixture-covered cases is not trustworthy enough for
core parent styles.

## Operational Decision Rules

| Situation | Default decision |
|---|---|
| Core parent style quality matters. | Hand-author or presetize the parent, then use migration output as evidence and regression input. |
| Generated output is faithful but verbose. | Improve post-assembly SQI refinement or reusable presets; do not mutate assembly to chase concision. |
| Generated output is unfaithful. | Fix converter, engine, schema, or fixtures with oracle evidence before SQI work. |
| Fixture coverage is weak for affected behavior. | Expand selection and held-out fixtures before trusting output-driven inference. |
| XML and measured output disagree. | Prefer measured output only for exercised behavior; retain XML fallback for latent branches, options, substitutes, and locale behavior. |
| A heuristic fix helps one style and risks another. | Require a narrow regression test, family/type guard, or move the behavior into declarative style data. |
| A style can be expressed with a generic spine plus few true outliers. | Prefer base templates, options, presets, and diff-based `type-variants` over many full type templates. |

The goal is not to make the converter clever enough to author every style.
The goal is to make it dependable as a source of explicit semantic candidates,
evidence, and controlled automation while preserving a path to high-quality
human-readable styles.

## Historical Analysis: XML Compiler

**How it works:** Parse CSL 1.0 XML, inline macros, upsample nodes to an
intermediate representation, then compile into Citum's `TemplateComponent`
model. Post-processing and fixups repair known structural mismatches.

### Strengths

1. **Semantic coverage** - XML contains the author's declared rules, including
   rare reference types and branches fixtures may not exercise.
2. **Options extraction** - Name formatting, et-al rules, initialize-with, date
   forms, page-range-format, and processing mode are usually recoverable from
   attributes and macro structure.
3. **Determinism** - Same XML input produces the same candidate output.
4. **Provenance** - Failures can be traced through the parsed CSL node and IR
   pipeline.
5. **Latent behavior** - Substitute rules, disambiguation hints, locale terms,
   and subsequent-author-substitute exist whether or not test data triggers
   them.

### Weaknesses

1. **Model mismatch** - CSL 1.0 is procedural: macros, conditionals, and groups
   with implicit suppression. Citum is declarative and typed.
2. **Type-conditioned flattening** - Macros such as APA's `source` contain many
   `choose/if/else` paths that produce different component sequences per
   reference type. Preserving XML order is not enough; the converter must infer
   which structures belong to which types.
3. **Group semantics** - CSL groups suppress delimiters when children are empty.
   Citum templates do not have the same implicit behavior, so naive conversion
   can create phantom punctuation or components.
4. **Heuristic fragility** - Reordering, grouping, media/type repairs, and
   bibliography flattening can become style-family heuristics unless tightly
   tested.
5. **Quality ceiling** - XML compilation can be faithful enough to render but
   still produce YAML no human would choose to maintain.

## Historical Analysis: Output-Driven Inference

**How it works:** Run citeproc-js against fixture references, parse rendered
output into components, cross-reference those components with input data, and
generate or score Citum templates from observed behavior.

### Strengths

1. **Direct fidelity target** - Oracle output is the executable comparator for
   CSL-derived behavior.
2. **Resolved conditionals** - citeproc-js has already selected branches,
   suppressed empty groups, and applied macro interactions for the exercised
   reference.
3. **Observed structure** - Component order, delimiters, formatting, and
   type-specific suppression can be derived from actual output.
4. **Human-readable bias** - The result tends to resemble the style a human
   would write from examples.

### Weaknesses

1. **Coverage-bound** - It only knows behavior present in selection or held-out
   fixtures.
2. **Metadata ambiguity** - Output text does not always reveal whether a token
   came from a contributor, editor, publisher, place, locale term, or literal.
3. **Cannot replace XML options** - Output cannot reliably identify
   initialize-with, name-as-sort-order, et-al thresholds, or latent substitute
   behavior.
4. **Locale conflation** - Rendered terms such as `pp.` may be locale terms,
   hardcoded affixes, or style-specific literals.
5. **Compensating errors** - If the Citum engine has a bug, inference can learn
   a template that compensates for the bug rather than exposing it.

### Current Result

The output-driven parser and inferrer have already reduced several historical
risks: component parsing is field-aware, delimiter inference uses filtered
voting, HTML formatting is inspected, and fixture coverage is broader than the
original 16-item set. This validates the hybrid approach, but it does not make
output-driven inference a complete replacement for XML or hand-authored styles.

## Historical Analysis: Hand-Authored Parent Styles

**How it works:** A human or domain-assisted author writes Citum YAML directly
from a style guide, examples, and oracle validation.

### Strengths

1. **Highest style quality** - The author can choose the clean generic spine,
   presets, and meaningful type outliers directly.
2. **Best authority handling** - Style guides and publisher instructions outrank
   converter artifacts when they are clearer than CSL XML.
3. **No inference ceiling** - Rare types, locale behavior, and edge cases can be
   handled deliberately.

### Weaknesses

1. **Does not scale to every dependent style** - It is appropriate for high
   leverage parent styles, not all migrated styles.
2. **Requires domain judgment** - The author must understand both citation
   behavior and Citum's declarative model.
3. **Still needs executable validation** - Human-authored styles must pass the
   same rendering and quality gates.

## Strategic Recommendation

The durable strategy is hybrid:

1. **Keep XML for options and latent semantics.** This is the reliable part of
   automatic migration and should not be discarded.
2. **Use measured output for exercised template structure.** Let citeproc-js
   settle concrete branch behavior where fixtures cover the case.
3. **Retain XML compilation as a fallback and evidence source.** It remains the
   only automatic path to rare unobserved branches.
4. **Author or presetize top parent styles.** The core portfolio should not be
   constrained by converter artifacts when a cleaner declarative style is known.
5. **Separate semantic assembly from SQI refinement.** Keep templates explicit
   until final refinement can diff, hoist, prune, and compact safely.
6. **Treat oracle and workflow tests as gates.** SQI improvements that regress
   fidelity are rejected.

## Validation and Risk Controls

Current and future migration work should be evaluated with layered checks:

1. Targeted converter tests for assembly boundaries, template diffing, measured
   selection, and SQI refinement.
2. Oracle/workflow tests for representative legacy styles, especially parent
   styles and styles whose generated output looks suspicious.
3. Expanded selection and held-out fixtures when changing output-driven
   inference or measured selection.
4. SQI reports for maintainability regressions after fidelity is green.
5. Manual review for high-impact parent styles, because clean Citum style design
   is an authoring problem as well as a converter problem.

## Files Referenced

- [`../../crates/citum-migrate/src/template_compiler/`](../../crates/citum-migrate/src/template_compiler/) - XML template compiler.
- [`../../crates/citum-migrate/src/options_extractor/`](../../crates/citum-migrate/src/options_extractor/) - XML-derived options pipeline.
- [`../../crates/citum-migrate/src/measured_citation.rs`](../../crates/citum-migrate/src/measured_citation.rs) - Oracle-scored candidate selection.
- [`../../crates/citum-migrate/src/synthesis/`](../../crates/citum-migrate/src/synthesis/) - Bounded candidate mutation and held-out validation.
- [`../../crates/citum-migrate/src/template_diff.rs`](../../crates/citum-migrate/src/template_diff.rs) - Diff-based `type-variants`.
- [`../../scripts/oracle.js`](../../scripts/oracle.js) - citeproc-js comparison harness.
- [`../../scripts/lib/component-parser.js`](../../scripts/lib/component-parser.js) - Field-aware output parser.
- [`../../scripts/lib/template-inferrer.js`](../../scripts/lib/template-inferrer.js) - Output-driven template inference.
- [`../../tests/fixtures/references-expanded.json`](../../tests/fixtures/references-expanded.json) - Selection fixture data.
- [`../../tests/fixtures/references-heldout.json`](../../tests/fixtures/references-heldout.json) - Held-out validation fixture data.
- [`../policies/SQI_REFINEMENT_PLAN.md`](../policies/SQI_REFINEMENT_PLAN.md) - Portfolio SQI policy.
