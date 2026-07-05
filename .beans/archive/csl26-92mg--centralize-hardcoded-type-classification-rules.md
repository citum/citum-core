---
# csl26-92mg
title: Centralize hardcoded type-classification rules
status: completed
type: task
priority: normal
tags:
    - types
    - engine
created_at: 2026-07-04T17:11:33Z
updated_at: 2026-07-05T13:24:05Z
parent: csl26-8m2p
---

Six inconsistent engine sites hardcode per-type presentation: dataset [Dataset] suffix string literal in get_effective_rendering; parent_short_title gates on ref_type.contains("article"); type_class_matches hardcodes legal/classical lists incl. contains("ancient"); get_title_category_rendering legacy type tables; SimpleVariable::Url DOI synthesis only for dataset; chapter silently matching entry-dictionary type-variants. Centralize into one classification table and replace the [Dataset] literal with a schema option. docs/architecture/audits/2026-07-04_CITUM_ENGINE_REVIEW_PART2.md finding 14.

## Spec

Design surfaced as a spec before implementation: docs/specs/TYPE_CLASSIFICATION_CENTRALIZATION.md (Status: Active — all four open decisions resolved during PR #1008 review). Part A (centralization of sites 2-6 into values/type_class.rs) implemented on the same PR; Part A produces byte-identical output (898/898 citum-engine tests pass, APA oracle unchanged vs. main). Part B (TypeLabel component replacing the [Dataset] suffix hack) is a follow-up PR.

## Summary of Changes

Part A implemented: new `crates/citum-engine/src/values/type_class.rs` module centralizes reference-type classification (title category for Primary/ContainerTitle/ParentSerial, TypeClass legal/classical membership, serial-parent gating, chapter/entry-dictionary selector alias, dataset DOI-URL synthesis). All five non-[Dataset] audit sites now call through it; the dead `contains("ancient")` fuzzy match (confirmed to match nothing `ref_type()` ever produces) is replaced with an explicit member list. Behavior-preserving: 898/898 citum-engine tests pass, APA oracle output unchanged vs. main, full workspace pre-commit gate (1774/1774 tests, fmt, clippy) clean.

Part B (the `[Dataset]` suffix hack / TypeLabel component) is tracked separately in [[csl26-rvhe]] since it is user-visible and schema-version-bumping, not mechanical centralization.
