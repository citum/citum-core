---
# csl26-2tjp
title: Add SortPreset to citum-schema
status: in-progress
type: feature
created_at: 2026-02-27T22:50:59Z
updated_at: 2026-02-27T22:50:59Z
---

Add a SortPreset enum to crates/citum-schema/src/presets.rs (parallel to ContributorPreset/DatePreset) that expands to a Sort struct. Named presets for the common sort patterns across citation styles (author-date-title, author-title-date, citation-number). Update the bibliography.sort block in styles/chicago-author-date-classic.yaml to use sort: author-date-title. Update the Sort struct in options/processing.rs to support untagged enum (Preset | Explicit) like DateConfigEntry. Refs: csl26-u5de
