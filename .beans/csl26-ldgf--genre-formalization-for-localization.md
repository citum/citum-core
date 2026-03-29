---
# csl26-ldgf
title: Genre formalization for localization
status: draft
type: feature
priority: normal
created_at: 2026-03-29T10:41:12Z
updated_at: 2026-03-29T17:32:00Z
---

The genre field is currently Option<String>, which cannot be localized. Formalize genre as an enum with locale-keyed labels so styles can render material-type descriptors (letter, photograph, etc.) in the target locale. Cross-cutting: affects all reference types. Triggered by archival spec review (csl26-jgt4).

## Decision (2026-03-29)

Keep genre and medium as Option<String> with a documented controlled vocabulary.
Rationale in docs/policies/ENUM_VOCABULARY_POLICY.md (Genre and Medium section).

Delivered:
- Canonical value table: docs/reference/GENRE_VALUES.md
- Policy: docs/policies/ENUM_VOCABULARY_POLICY.md
- types.rs doc comments reference the policy

Follow-up beans:
- csl26-qqfa: normalize fixture values to kebab-case
- csl26-0ueb: modularize types.rs
