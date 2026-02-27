---
# csl26-2tjp
title: Add SortPreset to citum-schema
status: completed
type: feature
priority: normal
created_at: 2026-02-27T22:50:59Z
updated_at: 2026-02-27T22:53:41Z
---

Add a SortPreset enum to crates/citum-schema/src/presets.rs (parallel to ContributorPreset/DatePreset) that expands to a Sort struct. Named presets for the common sort patterns across citation styles (author-date-title, author-title-date, citation-number). Update the bibliography.sort block in styles/chicago-author-date-classic.yaml to use sort: author-date-title. Update the Sort struct in options/processing.rs to support untagged enum (Preset | Explicit) like DateConfigEntry. Refs: csl26-u5de

## Summary of Changes\n\nAdded SortPreset enum (AuthorDateTitle, AuthorTitleDate, CitationNumber) to presets.rs. Wrapped Sort in SortEntry untagged enum (Preset | Explicit) in options/processing.rs, exported from options/mod.rs. Updated engine sorting.rs, migrate extractor, and tests. chicago-author-date-classic.yaml now uses `sort: author-date-title`.
