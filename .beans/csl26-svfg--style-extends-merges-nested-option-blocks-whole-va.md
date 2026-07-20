---
# csl26-svfg
title: 'Style extends: merges nested option blocks whole-value, not field-level'
status: draft
type: task
priority: normal
tags:
    - style
    - architecture
created_at: 2026-07-20T17:50:16Z
updated_at: 2026-07-20T17:50:26Z
---

Style inheritance (extends:) merges BibliographyOptions/CitationOptions/Config fields whole-value-replace, including nested struct fields like dates (crates/citum-schema-style/src/style/overlay.rs:340-344, same merge_options! macro shape as the runtime global/citation/bibliography scope merge in crates/citum-schema-style/src/options/mod.rs). A child style that wants to add one field on top of an inherited nested config (e.g. note-wrap alongside era-labels/approximation-marker) has no partial-override path -- it must redeclare the entire block. This forced gb-t-7714-2025-author-date/numeric/note to each carry a duplicated bibliography.options.dates copy in PR #1068 rather than one shared change in gb-t-7714-2025-base.yaml, and will recur for any style family with a shared base and scope-level nested option blocks (dates confirmed; contributors/titles/locators likely share the same shape -- worth auditing). Examine whether field-level partial merge on inherit is desirable in general (vs today's unambiguous whole-block-or-nothing), and if so what it would take: a deep-merge variant of merge_options! for nested-struct fields, an opt-in merge: deep annotation per field, or just a documented convention. Use the GB/T duplication as the motivating example, not the scope -- this is about the general inheritance mechanism, not consolidating GB/T's copies specifically.
