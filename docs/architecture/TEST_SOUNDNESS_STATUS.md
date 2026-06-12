# Citum Test Soundness Status

> **Living document** — the canonical index of test-quality state across
> `citum-core`. One row per behaviour spec: when its tests were last audited
> for soundness, the verdict counts, any open spec defects the audit surfaced,
> and what remains to do.
>
> **Last updated:** 2026-06-12
>
> **How to refresh:** run the [`test-soundness-review`](../../.skills/test-soundness-review/SKILL.md)
> skill against a spec and its tests. The skill reads this ledger first and
> upserts the spec's row here. Do not hand-edit verdict counts — let the skill
> own them.
>
> **What "soundness" means here** is defined once in
> [`docs/guides/CODING_STANDARDS.md`](../guides/CODING_STANDARDS.md)
> § "What makes a test worth keeping". This ledger tracks conformance to that
> contract; it does not redefine it.

## Status vocabulary

| Status | Meaning |
|--------|---------|
| `todo` | Never audited for soundness. |
| `audited` | Reviewed; findings recorded and still open. |
| `addressed` | Reviewed and the findings have been fixed. |
| `needs-rework` | Blocked on a spec decision — a `blocking` spec defect must be resolved before tests can be honestly classified. |

`Tests (G/S/B/R)` = good / suspicious / broken / redundant counts from the most
recent audit. `R` (redundant) flags low-value tests — near-duplicates,
tautologies, coverage theatre — whose usual fix is deletion, not rewrite.

## Backfill note (2026-06-12)

This ledger was seeded from the [`docs/specs/`](../specs/) inventory at
creation. Every spec starts `todo` except where there is concrete evidence of
prior soundness work in git:

- **DISAMBIGUATION.md** — `audited`. Commit `aa7af758`
  ("test(engine): harden disambiguation test soundness") was a targeted
  soundness pass; counts below are marked unknown (`?`) because that work
  predates this ledger and was not recorded as a verdict table. A fresh audit
  should replace the `?`s.

Soundness-*adjacent* work also exists but is **not** treated as an audit here,
to avoid overclaiming: the behavior-coverage reports for the `citations`,
`document`, and `i18n` engine suites (see
[`engine-behavior-reporting`](../../.claude/skills/engine-behavior-reporting/SKILL.md))
made those suites behaviour-oriented and readable, but did not classify each
test against its spec. Those specs remain `todo`.

## Ledger

| Spec / Module | Last reviewed | Tests (G/S/B/R) | Open spec issues | Status | Notes |
|---------------|---------------|-----------------|------------------|--------|-------|
| [ABBREVIATION_MAP.md](../specs/ABBREVIATION_MAP.md) | — | — | — | todo | — |
| [ANNOTATED_BIBLIOGRAPHY.md](../specs/ANNOTATED_BIBLIOGRAPHY.md) | — | — | — | todo | — |
| [APA_SQI_ALIGNMENT_AND_PRESET_REFACTOR.md](../specs/APA_SQI_ALIGNMENT_AND_PRESET_REFACTOR.md) | — | — | — | todo | — |
| [ARCHIVAL_UNPUBLISHED_SUPPORT.md](../specs/ARCHIVAL_UNPUBLISHED_SUPPORT.md) | — | — | — | todo | — |
| [ARTICLE_JOURNAL_NO_PAGE_FALLBACK.md](../specs/ARTICLE_JOURNAL_NO_PAGE_FALLBACK.md) | — | — | — | todo | — |
| [BIBLIOGRAPHY_GROUPING.md](../specs/BIBLIOGRAPHY_GROUPING.md) | — | — | — | todo | — |
| [BIBLIOGRAPHY_RENDERING_PIPELINE.md](../specs/BIBLIOGRAPHY_RENDERING_PIPELINE.md) | — | — | — | todo | — |
| [CHICAGO_18_COVERAGE.md](../specs/CHICAGO_18_COVERAGE.md) | — | — | — | todo | — |
| [CITATION_BIBLIOGRAPHY_OPTION_SPLIT.md](../specs/CITATION_BIBLIOGRAPHY_OPTION_SPLIT.md) | — | — | — | todo | — |
| [CITATION_CLUSTER_RENDERING.md](../specs/CITATION_CLUSTER_RENDERING.md) | — | — | — | todo | — |
| [CITE_SITE_COMPOUND_GROUPING.md](../specs/CITE_SITE_COMPOUND_GROUPING.md) | — | — | — | todo | — |
| [CLI_UX_REDESIGN.md](../specs/CLI_UX_REDESIGN.md) | — | — | — | todo | — |
| [CONFIG_ONLY_PROFILE_OVERRIDES.md](../specs/CONFIG_ONLY_PROFILE_OVERRIDES.md) | — | — | — | todo | — |
| [CROSS_ENTRY_FIDELITY.md](../specs/CROSS_ENTRY_FIDELITY.md) | — | — | — | todo | — |
| [DATE_MODEL.md](../specs/DATE_MODEL.md) | — | — | — | todo | — |
| [DISAMBIGUATION.md](../specs/DISAMBIGUATION.md) | 2026-06-12 | 24/0/0/0 | — | addressed | Added transliterated-name collision gap test; moved 1 out-of-scope wrapper out of the disambiguation behavior group. |
| [DISTRIBUTED_RESOLVER.md](../specs/DISTRIBUTED_RESOLVER.md) | — | — | — | todo | — |
| [DJOT_RICH_TEXT.md](../specs/DJOT_RICH_TEXT.md) | — | — | — | todo | — |
| [DOCUMENT_INPUT_PARSER_BOUNDARY.md](../specs/DOCUMENT_INPUT_PARSER_BOUNDARY.md) | — | — | — | todo | — |
| [EDTF_ERA_LABEL_PROFILES.md](../specs/EDTF_ERA_LABEL_PROFILES.md) | — | — | — | todo | — |
| [EDTF_HISTORICAL_ERA_RENDERING.md](../specs/EDTF_HISTORICAL_ERA_RENDERING.md) | — | — | — | todo | — |
| [EMBEDDED_JS_TEMPLATE_INFERENCE.md](../specs/EMBEDDED_JS_TEMPLATE_INFERENCE.md) | — | — | — | todo | — |
| [EMBEDDED_ROOT_WRAPPER_MIGRATION.md](../specs/EMBEDDED_ROOT_WRAPPER_MIGRATION.md) | — | — | — | todo | — |
| [ENGINE_MIGRATE_COEVOLUTION_WAVE.md](../specs/ENGINE_MIGRATE_COEVOLUTION_WAVE.md) | — | — | — | todo | — |
| [EXPLICIT_DEFAULT_SORTING.md](../specs/EXPLICIT_DEFAULT_SORTING.md) | — | — | — | todo | — |
| [FIDELITY_RICH_INPUTS.md](../specs/FIDELITY_RICH_INPUTS.md) | — | — | — | todo | — |
| [FORWARD_COMPATIBILITY.md](../specs/FORWARD_COMPATIBILITY.md) | — | — | — | todo | — |
| [GENDERED_LOCALE_TERMS.md](../specs/GENDERED_LOCALE_TERMS.md) | — | — | — | todo | — |
| [GENERALIZED_RELATIONAL_CONTAINER_MODEL.md](../specs/GENERALIZED_RELATIONAL_CONTAINER_MODEL.md) | — | — | — | todo | — |
| [INLINE_JOURNAL_DETAIL_GROUPING.md](../specs/INLINE_JOURNAL_DETAIL_GROUPING.md) | — | — | — | todo | — |
| [INPUT_REFERENCE_CLASS_DISCRIMINATOR.md](../specs/INPUT_REFERENCE_CLASS_DISCRIMINATOR.md) | — | — | — | todo | — |
| [INTEGRAL_NAME_MEMORY.md](../specs/INTEGRAL_NAME_MEMORY.md) | — | — | — | todo | — |
| [INTERACTIVE_STYLE_OVERRIDES.md](../specs/INTERACTIVE_STYLE_OVERRIDES.md) | — | — | — | todo | — |
| [JOURNAL_PROFILE_TAXONOMY_AUDIT.md](../specs/JOURNAL_PROFILE_TAXONOMY_AUDIT.md) | — | — | — | todo | — |
| [LANGUAGE_BINDINGS.md](../specs/LANGUAGE_BINDINGS.md) | — | — | — | todo | — |
| [LEGAL_CITATIONS.md](../specs/LEGAL_CITATIONS.md) | — | — | — | todo | — |
| [LOCALE_MESSAGES.md](../specs/LOCALE_MESSAGES.md) | — | — | — | todo | — |
| [LOCATOR_RENDERING.md](../specs/LOCATOR_RENDERING.md) | — | — | — | todo | — |
| [MIGRATE_RESEARCH_RICH_INPUTS.md](../specs/MIGRATE_RESEARCH_RICH_INPUTS.md) | — | — | — | todo | — |
| [MIGRATION_TAXONOMY_AWARE_WRAPPERS.md](../specs/MIGRATION_TAXONOMY_AWARE_WRAPPERS.md) | — | — | — | todo | — |
| [MIXED_CONDITION_NOTE_POSITION_TREES.md](../specs/MIXED_CONDITION_NOTE_POSITION_TREES.md) | — | — | — | todo | — |
| [MULTILINGUAL.md](../specs/MULTILINGUAL.md) | — | — | — | todo | — |
| [MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md](../specs/MULTILINGUAL_BIBLIOGRAPHY_PARTITIONING.md) | — | — | — | todo | — |
| [MULTILINGUAL_NAMES.md](../specs/MULTILINGUAL_NAMES.md) | — | — | — | todo | — |
| [NOCITE_BIBLIOGRAPHY_ONLY_ENTRIES.md](../specs/NOCITE_BIBLIOGRAPHY_ONLY_ENTRIES.md) | — | — | — | todo | — |
| [NON_STANDARD_NUMBERING_AND_LOCATOR_KINDS.md](../specs/NON_STANDARD_NUMBERING_AND_LOCATOR_KINDS.md) | — | — | — | todo | — |
| [NOTE_POSITION_AUDIT.md](../specs/NOTE_POSITION_AUDIT.md) | — | — | — | todo | — |
| [NOTE_SHORTENING_POLICY.md](../specs/NOTE_SHORTENING_POLICY.md) | — | — | — | todo | — |
| [NOTE_START_REPEATED_NOTE_POLICY.md](../specs/NOTE_START_REPEATED_NOTE_POLICY.md) | — | — | — | todo | — |
| [NOTE_STYLE_DOCUMENT_NOTE_CONTEXT.md](../specs/NOTE_STYLE_DOCUMENT_NOTE_CONTEXT.md) | — | — | — | todo | — |
| [NUMBERING_SEMANTICS.md](../specs/NUMBERING_SEMANTICS.md) | — | — | — | todo | — |
| [ORIGINAL_PUBLICATION_RELATION_SUPPORT.md](../specs/ORIGINAL_PUBLICATION_RELATION_SUPPORT.md) | — | — | — | todo | — |
| [OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md](../specs/OUTPUT_DRIVEN_TEMPLATE_SYNTHESIS.md) | — | — | — | todo | — |
| [PANDOC_MARKDOWN_CITATIONS.md](../specs/PANDOC_MARKDOWN_CITATIONS.md) | — | — | — | todo | — |
| [PERSONAL_COMMUNICATION_CITATION.md](../specs/PERSONAL_COMMUNICATION_CITATION.md) | — | — | — | todo | — |
| [PER_DOCUMENT_CONFIG_OVERRIDES.md](../specs/PER_DOCUMENT_CONFIG_OVERRIDES.md) | — | — | — | todo | — |
| [PROFILE_DOCUMENTARY_VERIFICATION.md](../specs/PROFILE_DOCUMENTARY_VERIFICATION.md) | — | — | — | todo | — |
| [PUNCTUATION_NORMALIZATION.md](../specs/PUNCTUATION_NORMALIZATION.md) | — | — | — | todo | — |
| [REPEATED_NOTE_CITATION_STATE_MODEL.md](../specs/REPEATED_NOTE_CITATION_STATE_MODEL.md) | — | — | — | todo | — |
| [ROLE_SUBSTITUTE_FALLBACK.md](../specs/ROLE_SUBSTITUTE_FALLBACK.md) | — | — | — | todo | — |
| [SCHEMA_SPLIT_AND_CONVERT_NAMESPACE.md](../specs/SCHEMA_SPLIT_AND_CONVERT_NAMESPACE.md) | — | — | — | todo | — |
| [SECONDARY_CONTRIBUTOR_ROLE_FORMATTING.md](../specs/SECONDARY_CONTRIBUTOR_ROLE_FORMATTING.md) | — | — | — | todo | — |
| [SENTENCE_INITIAL_LABELS.md](../specs/SENTENCE_INITIAL_LABELS.md) | — | — | — | todo | — |
| [SERVER_INTERACTIVE_API.md](../specs/SERVER_INTERACTIVE_API.md) | — | — | — | todo | — |
| [SHORT_NAME.md](../specs/SHORT_NAME.md) | — | — | — | todo | — |
| [SORTING.md](../specs/SORTING.md) | 2026-06-12 | 15/0/0/0 | — | addressed | Deleted 4 vacuous tests; fixed 2; added 5 gap tests; clarified 2 spec silences. |
| [STRONG_DOMAIN_TYPES_PHASE1.md](../specs/STRONG_DOMAIN_TYPES_PHASE1.md) | — | — | — | todo | — |
| [STYLE_ALIASING.md](../specs/STYLE_ALIASING.md) | — | — | — | todo | — |
| [STYLE_EDITIONS_AND_FAMILIES.md](../specs/STYLE_EDITIONS_AND_FAMILIES.md) | — | — | — | todo | — |
| [STYLE_PRESET_ARCHITECTURE.md](../specs/STYLE_PRESET_ARCHITECTURE.md) | — | — | — | todo | — |
| [STYLE_REGISTRY.md](../specs/STYLE_REGISTRY.md) | — | — | — | todo | — |
| [STYLE_TAXONOMY.md](../specs/STYLE_TAXONOMY.md) | — | — | — | todo | — |
| [TEMPLATE_V2.md](../specs/TEMPLATE_V2.md) | — | — | — | todo | — |
| [TEMPLATE_V3.md](../specs/TEMPLATE_V3.md) | — | — | — | todo | — |
| [TITLE_NAME_INFLECTION.md](../specs/TITLE_NAME_INFLECTION.md) | — | — | — | todo | — |
| [TITLE_TEXT_CASE.md](../specs/TITLE_TEXT_CASE.md) | — | — | — | todo | — |
| [TYPE_REFACTOR_v3.md](../specs/TYPE_REFACTOR_v3.md) | — | — | — | todo | — |
| [TYPE_SYSTEM_ARCHITECTURE.md](../specs/TYPE_SYSTEM_ARCHITECTURE.md) | — | — | — | todo | — |
| [UNICODE_BIBLIOGRAPHY_SORTING.md](../specs/UNICODE_BIBLIOGRAPHY_SORTING.md) | — | — | — | todo | — |
| [UNIFIED_SCOPED_OPTIONS.md](../specs/UNIFIED_SCOPED_OPTIONS.md) | — | — | — | todo | — |
| [WASM_SUPPORT.md](../specs/WASM_SUPPORT.md) | — | — | — | todo | — |

## How to read this as an agent

- **"What's the state of test quality?"** → scan the `Status` column. Anything
  `todo` is unaudited; `needs-rework` is blocked on a spec decision and is the
  highest-signal place to look.
- **"What should I audit next?"** → prefer high-risk `todo` specs (algorithmic
  or rendering behaviour: disambiguation, sorting, note position, locator,
  multilingual) over descriptive/architecture specs.
- **Don't trust a stale row.** If a spec changed materially since `Last
  reviewed`, the row is stale — re-run the skill rather than relying on the
  counts.
