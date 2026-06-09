---
# csl26-q1nc
title: warn on unknown fields in bibliography input
status: completed
type: bug
priority: high
created_at: 2026-06-09T20:27:36Z
updated_at: 2026-06-09T20:36:15Z
---

References with misspelled fields (e.g. lang: instead of language:, parent: instead of container:) are silently swallowed by the #[serde(flatten)] unknown_fields catch-alls on reference Deser structs. No warning anywhere — caused silent data loss in PR #895 CNE fixtures.

Fix: add unknown_reference_field_warnings(&Bibliography) -> Vec<Warning> in citum-engine/src/api/document.rs alongside unknown_reference_class_warnings; wire into session.rs + format_document central aggregation; CLI check (with --strict escalation) and render refs present it.

- [x] engine fn + export
- [x] central aggregation (session.rs, document.rs)
- [x] CLI check.rs + render refs presentation
- [x] tests in crates/citum-engine/tests/forward_compatibility.rs

## Summary of Changes

Added InputReference::unknown_fields() accessor (class_dispatch over all 18 typed classes; None for unknown-class). New engine API unknown_reference_field_warnings(&Bibliography) in api/document.rs, exported from api/mod.rs and wired into both central warning aggregation points (api/session.rs open-session path and format_document), so WASM/FFI/document adapters surface it too. CLI: citum check reports the warnings (hard error under --strict); citum render refs prints them to stderr. Two tests in crates/citum-engine/tests/forward_compatibility.rs (positive + clean negative). Verified against the broken PR #895 CNE fixture: all three lang/parent typos are now reported.
