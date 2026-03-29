---
# csl26-ldgf
title: Genre formalization for localization
status: draft
type: feature
created_at: 2026-03-29T10:41:12Z
updated_at: 2026-03-29T10:41:12Z
---

The genre field is currently Option<String>, which cannot be localized. Formalize genre as an enum with locale-keyed labels so styles can render material-type descriptors (letter, photograph, etc.) in the target locale. Cross-cutting: affects all reference types. Triggered by archival spec review (csl26-jgt4).
