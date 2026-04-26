# Specifications

Feature and design specifications for non-trivial Citum capabilities.
A spec captures the *what* and *why* before implementation begins.

## Spec Template

Copy this template when creating a new spec:

```markdown
# [Feature Name] Specification

**Status:** Draft | Active | Completed | Superseded
**Date:** YYYY-MM-DD
**Supersedes:** (optional path, if any)
**Related:** (policy, bean, or issue)

## Purpose
One paragraph: what feature this specifies and why.

## Scope
In scope. Explicitly out of scope.

## Design
(Core content — decisions, data models, examples.)

## Implementation Notes
(Non-normative hints, known constraints.)

## Acceptance Criteria
- [ ] Verifiable condition 1
- [ ] Verifiable condition 2

## Changelog
- DATE: Initial version.
```

## Workflow

Before creating or converting a spec, consult
[`../guides/DOCUMENT_CLASSIFICATION.md`](../guides/DOCUMENT_CLASSIFICATION.md)
to make sure the document should be a spec rather than architecture or policy.

1. Create `docs/specs/FEATURE_NAME.md` (Status: `Draft`) **before** writing
   implementation code.
2. Commit the spec. Get it merged.
3. Set Status to `Active` in the same commit as the first implementation.
4. Reference the spec path in the bean description.

## Specs

### Draft

| File | Feature |
|------|---------|
| [`PROFILE_DOCUMENTARY_VERIFICATION.md`](./PROFILE_DOCUMENTARY_VERIFICATION.md) | Verification model for profile styles using external authority |

### Active

| File | Feature |
|------|---------|
| [`ANNOTATED_BIBLIOGRAPHY.md`](./ANNOTATED_BIBLIOGRAPHY.md) | Document-scoped annotation overlay for bibliography rendering |
| [`APA_SQI_ALIGNMENT_AND_PRESET_REFACTOR.md`](./APA_SQI_ALIGNMENT_AND_PRESET_REFACTOR.md) | APA SQI alignment and preset-first cleanup |
| [`ARCHIVAL_UNPUBLISHED_SUPPORT.md`](./ARCHIVAL_UNPUBLISHED_SUPPORT.md) | ArchiveInfo and EprintInfo first-class source support |
| [`ARTICLE_JOURNAL_NO_PAGE_FALLBACK.md`](./ARTICLE_JOURNAL_NO_PAGE_FALLBACK.md) | External bibliography parameter for article-journal page fallback |
| [`CHICAGO_18_COVERAGE.md`](./CHICAGO_18_COVERAGE.md) | Chicago 18th and APA 8th high-fidelity coverage enhancement |
| [`CITATION_BIBLIOGRAPHY_OPTION_SPLIT.md`](./CITATION_BIBLIOGRAPHY_OPTION_SPLIT.md) | Strict schema split for citation and bibliography option scopes |
| [`CITE_SITE_COMPOUND_GROUPING.md`](./CITE_SITE_COMPOUND_GROUPING.md) | Cite-site compound grouping and position-aware overrides |
| [`DATE_MODEL.md`](./DATE_MODEL.md) | Refined date model for created vs. issued distinction |
| [`DOCUMENT_INPUT_PARSER_BOUNDARY.md`](./DOCUMENT_INPUT_PARSER_BOUNDARY.md) | Boundary between format parsers and shared processing pipeline |
| [`EDTF_ERA_LABEL_PROFILES.md`](./EDTF_ERA_LABEL_PROFILES.md) | Era label profiles and unspecified historical-year display |
| [`EDTF_HISTORICAL_ERA_RENDERING.md`](./EDTF_HISTORICAL_ERA_RENDERING.md) | Locale-backed rendering of valid historical EDTF years |
| [`EMBEDDED_JS_TEMPLATE_INFERENCE.md`](./EMBEDDED_JS_TEMPLATE_INFERENCE.md) | Embedded JS runtime for live template inference in migrator |
| [`ENGINE_MIGRATE_COEVOLUTION_WAVE.md`](./ENGINE_MIGRATE_COEVOLUTION_WAVE.md) | Engine-first co-evolution wave for style-fidelity fixes |
| [`FIDELITY_RICH_INPUTS.md`](./FIDELITY_RICH_INPUTS.md) | Fidelity pipeline support for relational benchmark inputs |
| [`GENDERED_LOCALE_TERMS.md`](./GENDERED_LOCALE_TERMS.md) | Multi-dimensional locale terms with grammatical gender support |
| [`GENERALIZED_RELATIONAL_CONTAINER_MODEL.md`](./GENERALIZED_RELATIONAL_CONTAINER_MODEL.md) | Recursive relational container model replacing flat variables |
| [`INLINE_JOURNAL_DETAIL_GROUPING.md`](./INLINE_JOURNAL_DETAIL_GROUPING.md) | Inline article-journal detail blocks with mixed delimiters |
| [`LANGUAGE_BINDINGS.md`](./LANGUAGE_BINDINGS.md) | Multi-language type bindings for canonical data shapes |
| [`LOCALE_MESSAGES.md`](./LOCALE_MESSAGES.md) | ICU MF2 parameterized message system replacing flat YAML terms |
| [`LOCATOR_RENDERING.md`](./LOCATOR_RENDERING.md) | Style-level LocatorConfig replacing per-template locator fields |
| [`MIGRATION_TAXONOMY_AWARE_WRAPPERS.md`](./MIGRATION_TAXONOMY_AWARE_WRAPPERS.md) | Taxonomy-aware wrapper derivation during style migration |
| [`MIGRATE_RESEARCH_RICH_INPUTS.md`](./MIGRATE_RESEARCH_RICH_INPUTS.md) | Bounded rich-input workflow for migrate-research passes |
| [`MIXED_CONDITION_NOTE_POSITION_TREES.md`](./MIXED_CONDITION_NOTE_POSITION_TREES.md) | Migration of legacy choose trees with mixed position predicates |
| [`NON_STANDARD_NUMBERING_AND_LOCATOR_KINDS.md`](./NON_STANDARD_NUMBERING_AND_LOCATOR_KINDS.md) | Representation of domain-specific numbering and locator labels |
| [`NOTE_POSITION_AUDIT.md`](./NOTE_POSITION_AUDIT.md) | Audit layer for note-style repeated-citation behavior |
| [`NOTE_SHORTENING_POLICY.md`](./NOTE_SHORTENING_POLICY.md) | Normative contract for repeated-note and shortened-note behavior |
| [`NOTE_START_REPEATED_NOTE_POLICY.md`](./NOTE_START_REPEATED_NOTE_POLICY.md) | Repeated-note behavior at note start vs. internal positions |
| [`NOTE_STYLE_DOCUMENT_NOTE_CONTEXT.md`](./NOTE_STYLE_DOCUMENT_NOTE_CONTEXT.md) | Document-level note processing and note-in-note participating |
| [`NUMBERING_SEMANTICS.md`](./NUMBERING_SEMANTICS.md) | Canonical semantics for numbering, report, and part fields |
| [`ORIGINAL_PUBLICATION_RELATION_SUPPORT.md`](./ORIGINAL_PUBLICATION_RELATION_SUPPORT.md) | Universal original publication metadata support across all types |
| [`PERSONAL_COMMUNICATION_CITATION.md`](./PERSONAL_COMMUNICATION_CITATION.md) | Rendering of personal communications across style families |
| [`REPEATED_NOTE_CITATION_STATE_MODEL.md`](./REPEATED_NOTE_CITATION_STATE_MODEL.md) | Style-driven repeated-citation model for note processing |
| [`ROLE_SUBSTITUTE_FALLBACK.md`](./ROLE_SUBSTITUTE_FALLBACK.md) | Normative behavior for role-aware contributor fallback chains |
| [`SCHEMA_SPLIT_AND_CONVERT_NAMESPACE.md`](./SCHEMA_SPLIT_AND_CONVERT_NAMESPACE.md) | Crate-level schema split and CLI conversion namespace |
| [`SECONDARY_CONTRIBUTOR_ROLE_FORMATTING.md`](./SECONDARY_CONTRIBUTOR_ROLE_FORMATTING.md) | Consistent rendering and verification for secondary contributor roles |
| [`SENTENCE_INITIAL_LABELS.md`](./SENTENCE_INITIAL_LABELS.md) | Sentence-initial capitalization for localized labels |
| [`STRONG_DOMAIN_TYPES_PHASE1.md`](./STRONG_DOMAIN_TYPES_PHASE1.md) | Replacing primitive String aliases with dedicated domain types |
| [`STYLE_PRESET_ARCHITECTURE.md`](./STYLE_PRESET_ARCHITECTURE.md) | Two-level configuration reuse for bases and profiles |
| [`STYLE_REGISTRY.md`](./STYLE_REGISTRY.md) | Serde-driven StyleRegistry replacing hardcoded slices |
| [`STYLE_TAXONOMY.md`](./STYLE_TAXONOMY.md) | Citum style taxonomy based on semantic class and implementation |
| [`TEMPLATE_V2.md`](./TEMPLATE_V2.md) | Simplified Template Schema v2 with group-first composition |
| [`TITLE_TEXT_CASE.md`](./TITLE_TEXT_CASE.md) | Modeling and applying title-like text-case transformations |
| [`TYPE_REFACTOR_v3.md`](./TYPE_REFACTOR_v3.md) | Unified type system refactor for high-fidelity work modeling |
| [`UNICODE_BIBLIOGRAPHY_SORTING.md`](./UNICODE_BIBLIOGRAPHY_SORTING.md) | Locale-aware Unicode collation for bibliography sorting |
| [`UNIFIED_SCOPED_OPTIONS.md`](./UNIFIED_SCOPED_OPTIONS.md) | Typed scoped options replacing flat author-facing contracts |

### Completed

| File | Feature |
|------|---------|
| [`JOURNAL_PROFILE_TAXONOMY_AUDIT.md`](./JOURNAL_PROFILE_TAXONOMY_AUDIT.md) | Audit of journal-profile taxonomy and authority rules |
| [`PANDOC_MARKDOWN_CITATIONS.md`](./PANDOC_MARKDOWN_CITATIONS.md) | Pandoc-style citation marker support for Markdown documents |

### Superseded

| File | Feature |
|------|---------|
| [`CONFIG_ONLY_PROFILE_OVERRIDES.md`](./CONFIG_ONLY_PROFILE_OVERRIDES.md) | Alternative configuration-only profile override model |
