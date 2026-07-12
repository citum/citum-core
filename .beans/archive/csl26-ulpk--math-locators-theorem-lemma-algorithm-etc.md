---
# csl26-ulpk
title: Math locators (theorem, lemma, algorithm, etc.)
status: completed
type: feature
priority: low
tags:
    - schema
    - locale
created_at: 2026-07-12T15:35:09Z
updated_at: 2026-07-12T22:46:35Z
parent: csl26-kcda
---

CSL schema#440: add locator terms for mathematical/formal-proof citation:
theorem, lemma, algorithm, problem, definition, proposition, corollary
(long/short forms per the issue, e.g. "Thm."/"Thms.").

Cohesive, self-contained set — no dependency on other gaps in this triage.

- [x] Add terms to crates/citum-schema-style/embedded/locales/en-US.yaml

## Summary of Changes

Added 7 locale terms (algorithm, corollary, definition, lemma, problem, proposition, theorem) to `crates/citum-schema-style/embedded/locales/en-US.yaml`, each with long/short singular/plural forms, matching the existing locator-term entries (page, chapter, paragraph, etc.).

No Rust/schema changes were needed: `LocatorType` already had all 7 variants and `english_locator_aliases()` already parsed them from free text. Verified via a temporary test that the new yaml terms populate `Locale.locators` for each `LocatorType` variant and that alias parsing (e.g. "thm. 3") still resolves correctly.
