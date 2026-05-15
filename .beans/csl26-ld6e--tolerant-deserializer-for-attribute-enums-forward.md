---
# csl26-ld6e
title: Tolerant deserializer for attribute enums (forward compat)
status: todo
type: feature
priority: high
created_at: 2026-05-15T14:48:03Z
updated_at: 2026-05-15T14:48:03Z
blocked_by:
    - csl26-2a0b
---

Promote forward-compat rows 01, 02, 03, 04 in crates/citum-engine/tests/snapshots/forward_compat_gaps.snap from observed=HardFail to observed=SoftDegrade.

Wrap derived Deserialize on attribute enums with an Unknown(String) fallback that emits a CompatibilityWarning. Scope:
- ContributorRole (crates/citum-schema-style/src/template.rs)
- MonographType, SerialComponentType, CollectionType, MonographComponentType (crates/citum-schema-data/src/reference/types/structural.rs)
- TermForm, GrammaticalGender (crates/citum-schema-style/src/locale/types.rs)
- DateForm (crates/citum-schema-style/src/template.rs)
- NumberingType (crates/citum-schema-data/src/reference/)

Excludes InputReference::class — that one is a dispatch discriminator (see follow-up bean for that design).

Spec: docs/specs/FORWARD_COMPATIBILITY.md
Snapshot rows: 01-attr-enum-template, 02-attr-enum-data, 03-locale-form, 04-date-form
