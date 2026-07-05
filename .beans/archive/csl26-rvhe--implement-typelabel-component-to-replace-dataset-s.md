---
# csl26-rvhe
title: Implement TypeLabel component to replace [Dataset] suffix hack
status: completed
type: task
priority: normal
created_at: 2026-07-05T13:23:57Z
updated_at: 2026-07-05T14:42:25Z
parent: csl26-8m2p
blocked_by:
    - csl26-92mg
---

Part B of docs/specs/TYPE_CLASSIFICATION_CENTRALIZATION.md: add TemplateComponent::TypeLabel (localized reference-type term with genre/medium fallback), rewrite apa-7th.yaml's dataset variant to drop the suffix: " [Dataset]." literal and use type-label + wrap: brackets instead, remove the engine de-dup at render/component.rs, regenerate schemas (STYLE_SCHEMA_VERSION minor bump via feat commit), update fixtures in tests/domain_fixtures.rs and tests/bibliography.rs. Confirm the exact label term text with the user before finalizing.

## Summary of Changes

Implemented as designed, with one correction during self-review: an early draft used `wrap.inner-prefix: "Version "` in apa-7th.yaml, which reintroduced the exact English-literal-in-a-style problem this whole effort exists to remove. Fixed by adding a new `GeneralTerm::Version` locale term (already present as dead data in en-US.yaml) and using `term: version` + `text-case: capitalize-first` instead.

- New `TemplateComponent::TypeLabel` (citum-schema-style/src/template.rs): resolves genre -> medium -> locale term (via new `Locale::type_terms`, an explicit allowlist cross-referenced against ref_type()'s real vocabulary -- not a blanket capture, since the terms: block also carries unrelated dead data like version/printing/era terms that must not leak in).
- New `Dataset.genre` field + accessor wiring (citum-schema-data), replacing the old baked-title synthesis ("[genre] (Version X)") in scholarly.rs with the real field plus the pre-existing (but previously dataset-title-only-used) `variable: version` component.
- apa-7th.yaml dataset type-variant rewritten: title suffix hack removed; type-label + version now rendered via a nested group so punctuation is correct whether or not a title/version is present.
- Verified: 898/898 citum-engine tests, 1774/1774 full workspace tests, APA oracle unchanged vs. main (same single pre-existing mismatch), just pre-commit clean, schemas regenerated.

Spec docs/specs/TYPE_CLASSIFICATION_CENTRALIZATION.md updated to v1.2 (Acceptance Criteria checked off).
